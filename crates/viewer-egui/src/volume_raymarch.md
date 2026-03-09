# volume_raymarch.wgsl

## Purpose
WGSL shader for the `Volume 3D` mode. It raymarches the uploaded 3D texture through the Lenia volume, applies the transfer function, and composites color/opacity front-to-back.

## Components

### `vs_main`
- **Does**: Emits a full-screen triangle covering the viewport callback region.
- **Interacts with**: `VolumePaintCallback::paint` in `volume_wgpu.rs`

### `fs_main`
- **Does**: Unprojects the view ray, intersects the unit cube, samples the 3D volume texture, and composites the result.
- **Interacts with**: Uniforms and texture uploaded in `volume_wgpu.rs`

### `intersect_box`
- **Does**: Computes the parametric entry/exit range for the unit cube.
- **Interacts with**: `fs_main`

### `palette`
- **Does**: Maps normalized density values to the viewer color ramp.
- **Interacts with**: `fs_main`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `volume_wgpu.rs` | Uniform struct layout matches Rust-side `VolumeUniforms` exactly | Reordering or resizing uniforms |
| Users of `Volume 3D` mode | The shader treats the simulation domain as a cube in `[-1, 1]^3` | Changing the domain mapping |

## Notes
- This shader is the first filled-space renderer for the project. It should be treated as the baseline for future GPU-native simulation/display integration.
- The front-to-back compositing path updates the accumulated color/alpha as a full `vec4` because WGSL does not permit assigning into swizzles like `.rgb`.
