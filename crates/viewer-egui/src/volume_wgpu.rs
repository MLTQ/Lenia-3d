use crate::viewport3d::VolumeViewport;
use bytemuck::{Pod, Zeroable};
use eframe::{egui, egui_wgpu, wgpu};
use egui_wgpu::CallbackTrait;
use glam::Mat4;
use lenia_core::World3D;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

const SHADER: &str = include_str!("volume_raymarch.wgsl");

#[derive(Clone)]
pub struct VolumeWgpuRenderer {
    shared: Arc<Mutex<VolumeWgpuState>>,
}

struct VolumeWgpuState {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
    volume_texture: Option<wgpu::Texture>,
    volume_view: Option<wgpu::TextureView>,
    volume_size: [u32; 3],
}

#[derive(Clone)]
struct VolumeFrameData {
    volume_bytes: Arc<[u8]>,
    volume_size: [u32; 3],
    uniforms: VolumeUniforms,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct VolumeUniforms {
    inverse_view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    settings: [f32; 4],
}

impl VolumeWgpuRenderer {
    pub fn new(render_state: &egui_wgpu::RenderState) -> Self {
        let device = &render_state.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lenia-volume-raymarch-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lenia-volume-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            wgpu::BufferSize::new(std::mem::size_of::<VolumeUniforms>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lenia-volume-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lenia-volume-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("lenia-volume-uniforms"),
            contents: bytemuck::bytes_of(&VolumeUniforms::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lenia-volume-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Self {
            shared: Arc::new(Mutex::new(VolumeWgpuState {
                pipeline,
                bind_group_layout,
                sampler,
                uniform_buffer,
                bind_group: None,
                volume_texture: None,
                volume_view: None,
                volume_size: [0, 0, 0],
            })),
        }
    }

    pub fn paint_callback(
        &self,
        rect: egui::Rect,
        world: &World3D,
        viewport: &VolumeViewport,
    ) -> egui::PaintCallback {
        let aspect = (rect.width() / rect.height()).max(0.1);
        let (inverse_view_projection, camera_position) = viewport.view_projection_inverse(aspect);

        let frame = VolumeFrameData {
            volume_bytes: pack_volume_texture(world),
            volume_size: {
                let (depth, height, width) = world.shape();
                [width as u32, height as u32, depth as u32]
            },
            uniforms: VolumeUniforms {
                inverse_view_projection: inverse_view_projection.to_cols_array_2d(),
                camera_position,
                settings: [
                    viewport.visibility_floor,
                    viewport.opacity_scale,
                    viewport.step_count as f32,
                    0.0,
                ],
            },
        };

        egui_wgpu::Callback::new_paint_callback(
            rect,
            VolumePaintCallback {
                renderer: self.clone(),
                frame,
            },
        )
    }
}

struct VolumePaintCallback {
    renderer: VolumeWgpuRenderer,
    frame: VolumeFrameData,
}

impl CallbackTrait for VolumePaintCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut state = self.renderer.shared.lock().unwrap();
        state.ensure_volume_resources(device, &self.frame);
        state.upload(queue, &self.frame);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let state = self.renderer.shared.lock().unwrap();
        let Some(bind_group) = state.bind_group.as_ref() else {
            return;
        };

        let clip = info.clip_rect_in_pixels();
        render_pass.set_scissor_rect(
            clip.left_px.max(0) as u32,
            clip.top_px.max(0) as u32,
            clip.width_px.max(0) as u32,
            clip.height_px.max(0) as u32,
        );
        render_pass.set_pipeline(&state.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

impl VolumeWgpuState {
    fn ensure_volume_resources(&mut self, device: &wgpu::Device, frame: &VolumeFrameData) {
        if self.volume_size == frame.volume_size && self.bind_group.is_some() {
            return;
        }

        self.volume_size = frame.volume_size;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("lenia-volume-texture"),
            size: wgpu::Extent3d {
                width: frame.volume_size[0],
                height: frame.volume_size[1],
                depth_or_array_layers: frame.volume_size[2],
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lenia-volume-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.volume_texture = Some(texture);
        self.volume_view = Some(view);
        self.bind_group = Some(bind_group);
    }

    fn upload(&mut self, queue: &wgpu::Queue, frame: &VolumeFrameData) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&frame.uniforms));

        let Some(texture) = self.volume_texture.as_ref() else {
            return;
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &frame.volume_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame.volume_size[0]),
                rows_per_image: Some(frame.volume_size[1]),
            },
            wgpu::Extent3d {
                width: frame.volume_size[0],
                height: frame.volume_size[1],
                depth_or_array_layers: frame.volume_size[2],
            },
        );
    }
}

fn pack_volume_texture(world: &World3D) -> Arc<[u8]> {
    let bytes = world
        .view()
        .iter()
        .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect::<Vec<_>>();
    bytes.into()
}

impl Default for VolumeUniforms {
    fn default() -> Self {
        Self {
            inverse_view_projection: Mat4::IDENTITY.to_cols_array_2d(),
            camera_position: [0.0, 0.0, 3.0, 1.0],
            settings: [0.02, 2.0, 96.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pack_volume_texture;
    use lenia_core::World3D;

    #[test]
    fn pack_volume_texture_matches_world_len() {
        let world = World3D::zeros(4, 5, 6);
        let texture = pack_volume_texture(&world);
        assert_eq!(texture.len(), 4 * 5 * 6);
    }

    #[test]
    fn pack_volume_texture_clamps_values() {
        let mut world = World3D::zeros(2, 2, 2);
        world.set(0, 0, 0, 2.0);
        let texture = pack_volume_texture(&world);
        assert_eq!(texture[0], 255);
    }
}
