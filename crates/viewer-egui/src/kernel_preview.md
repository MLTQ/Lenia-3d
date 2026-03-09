# kernel_preview.rs

## Purpose
Builds and draws the lightweight kernel-preview UI for the egui viewer. This file converts the current 3D kernel into a 2D center slice plus a radial profile so kernel tuning is inspectable without touching the simulation viewport.

## Components

### `kernel_slice_image`
- **Does**: Generates a normalized colorized image of the kernel's central Z slice.
- **Interacts with**: `generate_kernel_3d` in `lenia-core`, texture upload in `app.rs`

### `kernel_radial_profile`
- **Does**: Averages the full 3D kernel into radius bins measured from the center.
- **Interacts with**: `generate_kernel_3d` in `lenia-core`, `draw_radial_kernel_plot`

### `draw_radial_kernel_plot`
- **Does**: Renders the radial profile as a small painter-based chart.
- **Interacts with**: `ViewerApp::draw_controls` in `app.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `app.rs` | `kernel_slice_image` returns an image sized to the kernel diameter | Changing image orientation or size |
| `app.rs` | `kernel_radial_profile` bins radii from `0..=radius_cells` | Changing bin semantics |

## Notes
- The preview intentionally uses the same palette as the planar world views so the kernel and field stay visually comparable.
