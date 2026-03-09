# viewport3d.rs

## Purpose
Owns the interactive camera state and egui integration for the 3D viewport. This file handles pointer input, camera math, and dispatch into the GPU raymarch renderer.

## Components

### `VolumeViewport`
- **Does**: Stores orbit camera and transfer-function settings for the volume renderer.
- **Interacts with**: `VolumeWgpuRenderer` in `volume_wgpu.rs`, `ViewerApp` in `app.rs`

### `VolumeViewport::draw`
- **Does**: Allocates the viewport region, applies camera input, and submits the GPU paint callback.
- **Interacts with**: `VolumeWgpuRenderer::paint_callback` in `volume_wgpu.rs`

### `VolumeViewport::view_projection_inverse`
- **Does**: Builds the inverse view-projection matrix and camera position consumed by the raymarch shader.
- **Interacts with**: Uniform generation in `volume_wgpu.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `app.rs` | `draw` renders the active 3D viewport or a clear fallback message | Changing the draw signature |
| `volume_wgpu.rs` | Camera uniforms reflect the current orbit settings | Changing camera conventions without updating shader math |

## Notes
- This file no longer performs CPU billboard rendering. The actual filled-space rendering now lives in the GPU callback path.
