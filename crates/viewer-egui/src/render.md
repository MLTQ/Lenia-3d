# render.rs

## Purpose
Converts 3D Lenia volumes into 2D images for the egui viewer and provides the shared color mapping used across viewer modes. This file keeps slice selection, projection logic, and palette choices out of the UI event code.

## Components

### `ViewMode`
- **Does**: Describes the supported true 3D, slice, and max-intensity projection views.
- **Interacts with**: `ViewerApp` in `app.rs`, `VolumeViewport` in `viewport3d.rs`

### `render_world_view`
- **Does**: Projects a `World3D` into a `ColorImage` for the planar slice/MIP modes.
- **Interacts with**: `World3D` in `lenia-core`, texture upload in `app.rs`
- **Rationale**: The viewer should be able to change planar projection logic without changing its UI state management.

### `colorize_value`
- **Does**: Maps normalized scalar values onto the viewer palette.
- **Interacts with**: `render_world_view` in this file, point colors in `viewport3d.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `app.rs` | `render_world_view` returns a correctly sized `ColorImage` for planar modes and skips the 3D mode via `requires_texture` | Changing image orientation or texture expectations |
| Tests in this file | Slice and MIP dimensions match world axes | Reordering axes |

## Notes
- The color map is deliberately higher-contrast than grayscale so faint structures read more clearly during exploration in both 2D and 3D views.
