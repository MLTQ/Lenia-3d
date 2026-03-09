use crate::volume_wgpu::VolumeWgpuRenderer;
use eframe::egui::{self, Color32, Sense};
use glam::{Mat4, Vec3};
use lenia_core::World3D;

#[derive(Clone, Debug)]
pub struct VolumeViewport {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub visibility_floor: f32,
    pub opacity_scale: f32,
    pub step_count: u32,
}

impl Default for VolumeViewport {
    fn default() -> Self {
        Self {
            yaw: 0.7,
            pitch: 0.5,
            distance: 3.2,
            visibility_floor: 0.02,
            opacity_scale: 2.0,
            step_count: 96,
        }
    }
}

impl VolumeViewport {
    pub fn reset_camera(&mut self) {
        self.yaw = 0.7;
        self.pitch = 0.5;
        self.distance = 3.2;
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        world: &World3D,
        renderer: Option<&VolumeWgpuRenderer>,
    ) {
        let available = ui.available_size();
        let desired = egui::vec2(available.x.max(320.0), available.y.max(320.0));
        let (rect, response) = ui.allocate_exact_size(desired, Sense::drag());
        self.handle_input(ui, &response);

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 10.0, Color32::from_rgb(5, 8, 15));

        if let Some(renderer) = renderer {
            painter.add(renderer.paint_callback(rect, world, self));
        } else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "wgpu volume renderer unavailable",
                egui::FontId::proportional(18.0),
                Color32::from_rgb(180, 184, 192),
            );
        }
    }

    pub fn view_projection_inverse(&self, aspect_ratio: f32) -> (Mat4, [f32; 4]) {
        let camera = self.camera_position();
        let view = Mat4::look_at_rh(camera, Vec3::ZERO, Vec3::Y);
        let projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);
        let inverse = (projection * view).inverse();
        (inverse, [camera.x, camera.y, camera.z, 1.0])
    }

    fn camera_position(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(
            self.distance * cos_pitch * sin_yaw,
            self.distance * sin_pitch,
            self.distance * cos_pitch * cos_yaw,
        )
    }

    fn handle_input(&mut self, ui: &egui::Ui, response: &egui::Response) {
        if response.dragged() {
            let delta = ui.input(|input| input.pointer.delta());
            self.yaw += delta.x * 0.01;
            self.pitch = (self.pitch - delta.y * 0.01).clamp(-1.45, 1.45);
        }

        if response.hovered() {
            let scroll = ui.input(|input| input.raw_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                self.distance = (self.distance - scroll * 0.0035).clamp(1.5, 8.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeViewport;

    #[test]
    fn camera_position_is_finite() {
        let viewport = VolumeViewport::default();
        let (_inverse, camera) = viewport.view_projection_inverse(1.0);
        assert!(camera.iter().all(|value| value.is_finite()));
    }
}
