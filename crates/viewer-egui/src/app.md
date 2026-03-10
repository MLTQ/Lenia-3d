# app.rs

## Purpose
Implements the interactive egui application for exploring 3D Lenia fields. This file owns single-channel and experimental multichannel simulation state, user controls, species loading, backend selection, kernel-family tuning, and dispatch between the true 3D viewport and the planar slice/MIP views.

## Components

### `ViewerApp`
- **Does**: Holds single-channel state, experimental multichannel state, simulation parameters, active backend, and viewer state.
- **Interacts with**: `SimulationBackend` implementations in `lenia-core`, `MultiChannelFftBackend` and multichannel helpers in `lenia-core`, species helpers in `lenia-core`, `render_world_view` in `render.rs`, `VolumeViewport` in `viewport3d.rs`, `VolumeWgpuRenderer` in `volume_wgpu.rs`, kernel preview helpers in `kernel_preview.rs`, `stamp_gaussian_blob_3d` in `lenia-core`

### `FoodSettings`
- **Does**: Stores periodic food-source configuration for the active world.
- **Interacts with**: `ViewerApp::apply_food_sources`, controls in `draw_controls`
- **Rationale**: Food reseeding is simulation state, not rendering state.

### `ViewerApp::step_once`
- **Does**: Advances either the single-channel world or the experimental multichannel world by one step, then refreshes the displayed scalar volume.
- **Interacts with**: `ReferenceBackend`, `FftBackend`, and `MultiChannelFftBackend` from `lenia-core`
- **Rationale**: Backend switching and multichannel bridging stay local to the app instead of leaking into rendering code.
- **Notes**: If the simulation extinguishes to an effectively zero-density field, this method immediately reseeds with a fresh randomized world so the viewer does not get stuck in an empty state.

### `ViewerApp::draw_controls`
- **Does**: Renders the side-panel controls for execution, backend choice, experimental multichannel mode, food seeding, species loading/scaling, non-destructive resizing, view selection, and Lenia parameters.
- **Interacts with**: `ViewMode` in `render.rs`, `LeniaParams`, `KernelMode`, `KernelCore`, and `MultiChannelParams` in `lenia-core`
- **Rationale**: The `Volume 3D` controls intentionally expose transfer-function settings instead of renderer-internal debug knobs so the CPU slice path and GPU volume path stay separate.

### `ViewerApp::apply_selected_species`
- **Does**: Loads the selected official 3D species preset at the requested integer scale by enlarging both the seed voxels and the kernel radius together, then chooses a recommended box size that fits the scaled organism without blindly multiplying the whole world edge by that scale.
- **Interacts with**: `scaled_params_for_preset`, `scaled_seed_shape_for_preset`, `seeded_world_for_preset_scaled`, and `SingleSpeciesPreset` in `lenia-core`

### `ViewerApp::apply_ndkc_starter`
- **Does**: Switches the app into experimental multichannel mode and seeds the starter NDKC-style world.
- **Interacts with**: `seed_ndkc_starter_world` and `MultiChannelParams::ndkc_starter_preset` in `lenia-core`

### `ViewerApp::rebuild_display_world_from_multichannel`
- **Does**: Converts the multichannel state into a single `World3D` for the existing renderers by taking the per-voxel maximum across channels.
- **Interacts with**: Slice/MIP rendering in `render.rs`, volume rendering in `viewport3d.rs`

### `ViewerApp::refresh_kernel_texture`
- **Does**: Uploads the current kernel center slice as an egui texture for the preview panel, using the selected multichannel rule when applicable.
- **Interacts with**: `kernel_slice_image` in `kernel_preview.rs`

### `ViewerApp::apply_food_sources`
- **Does**: Periodically stamps 3D Gaussian food blobs into the single-channel world using fixed or randomized sources.
- **Interacts with**: `stamp_gaussian_blob_3d` in `lenia-core`

### `ViewerApp::resize_world_preserving_state`
- **Does**: Changes the active world size without reseeding by centered padding/cropping the current state.
- **Interacts with**: `centered_resize_world`, `centered_resize_multichannel_world`
- **Rationale**: Changing the bounding box should not silently destroy a curated specimen.

### `ViewerApp::draw_viewport`
- **Does**: Chooses between the orbitable 3D viewport and the 2D texture-backed inspection modes.
- **Interacts with**: Texture state created by `refresh_texture`, `VolumeViewport::draw` in `viewport3d.rs`

### `world_stats`
- **Does**: Computes summary statistics for the current world.
- **Interacts with**: Labels in `draw_controls`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `main.rs` | `ViewerApp::new` returns a runnable egui app and opportunistically initializes the `wgpu` volume renderer from `CreationContext` | Renaming constructor or requiring different renderer setup |
| Users of the viewer | Play/pause, step, backend switching, species loading, experimental multichannel starter loading, extinction auto-reseeding, periodic food seeding, kernel-family switching, kernel previewing, 3D viewing, and view selection all function without external configuration | Removing controls or changing runtime behavior materially |

## Notes
- `Volume 3D` now uses a `wgpu` raymarch callback when the native app is running with the `eframe` `wgpu` backend enabled.
- Slice and MIP modes still go through the CPU texture path because they are the fastest debugging views for inspecting raw field data.
- Extinction is defined conservatively as `max_value <= 1e-6`, which tolerates tiny floating-point residue while still treating a visually dead field as dead.
- The side-panel is scrollable now because the kernel preview and expanded controls no longer fit reliably in a fixed-height column.
- The world-size ceiling is now `256` so scaled species and larger empty bounding volumes fit without a code change.
- Loading a species preset disables periodic food so the seeded organism evolves as a closed system unless the user re-enables food manually.
- Loading a species preset now also restores its preferred world size and forces the single-channel FFT backend so the seed runs under the conditions it was tuned against.
- Species loading now pauses playback and switches extinction behavior from auto-randomize to pause, so a curated specimen is not immediately replaced by exploratory noise if it dies.
- Species loading now exposes a scale control that replicates the official seed voxels and multiplies `radius_cells` by the same factor, but it no longer multiplies the whole world edge by that scale. The app now grows the box only to a recommended minimum that fits the scaled organism plus some breathing room.
- The single-channel editor now exposes an `Official Preset` button and `LENIA_BANDS` mode so the viewer can inspect or tweak the same kernel family used by the published 3D species data.
- Experimental multichannel mode currently renders a max-over-channels composite so the existing slice, MIP, and raymarch views can display it without a renderer rewrite.
- Experimental multichannel mode now steps through a cached FFT backend instead of the reference oracle, which keeps the UI interactive while preserving the reference path in core for parity tests.
- The world panel now offers both destructive resize and preserve-resize so users can experiment with different volume sizes without randomizing the current state.
