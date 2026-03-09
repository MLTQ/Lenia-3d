# volume_wgpu.rs

## Purpose
Implements the GPU-backed volume renderer used by the `Volume 3D` mode. This file owns the `wgpu` pipeline, 3D texture upload, callback integration with egui, and the uniforms needed by the raymarch shader.

## Components

### `VolumeWgpuRenderer`
- **Does**: Owns the shared GPU renderer state and creates egui paint callbacks for the current frame.
- **Interacts with**: `VolumeViewport` in `viewport3d.rs`, `ViewerApp` in `app.rs`, `eframe`'s `wgpu` render state
- **Rationale**: Keeping the renderer behind a single object lets the app stay agnostic about pipeline, texture, and bind-group lifetime.

### `VolumePaintCallback`
- **Does**: Bridges egui's paint-callback lifecycle to the renderer's GPU upload and draw steps.
- **Interacts with**: `egui_wgpu::CallbackTrait`, `VolumeWgpuState`
- **Rationale**: The callback owns per-frame upload data while reusing cached GPU resources across frames.

### `VolumeWgpuState`
- **Does**: Stores the persistent `wgpu` resources: pipeline, bind group layout, sampler, uniform buffer, and current 3D texture.
- **Interacts with**: `VolumePaintCallback::prepare` and `paint`

### `pack_volume_texture`
- **Does**: Converts the simulation volume into an `R8Unorm` texture payload for the shader.
- **Interacts with**: `World3D` in `lenia-core`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `viewport3d.rs` | `paint_callback` returns a valid `egui::PaintCallback` for the current world | Changing the callback construction API |
| Shader in `volume_raymarch.wgsl` | Uniform layout and texture format stay consistent with Rust-side uploads | Reordering uniforms or changing texture format |

## Notes
- This replaces the earlier CPU billboard renderer for `Volume 3D`.
- The texture upload is still CPU-to-GPU each frame; the long-term path is to render directly from a GPU simulation texture.
- The renderer depends on compiling `eframe` with its `wgpu` feature enabled.
