# app.rs

## Purpose
Implements the interactive egui application for exploring 3D Lenia fields. This file owns simulation state, user controls, backend selection, and dispatch between the true 3D viewport and the planar slice/MIP views.

## Components

### `ViewerApp`
- **Does**: Holds world state, simulation parameters, active backend, and viewer state.
- **Interacts with**: `SimulationBackend` implementations in `lenia-core`, `render_world_view` in `render.rs`, `VolumeViewport` in `viewport3d.rs`, `VolumeWgpuRenderer` in `volume_wgpu.rs`, `stamp_gaussian_blob_3d` in `lenia-core`

### `FoodSettings`
- **Does**: Stores periodic food-source configuration for the active world.
- **Interacts with**: `ViewerApp::apply_food_sources`, controls in `draw_controls`
- **Rationale**: Food reseeding is simulation state, not rendering state.

### `ViewerApp::step_once`
- **Does**: Advances the world by one step using the selected backend.
- **Interacts with**: `ReferenceBackend` and `FftBackend` from `lenia-core`
- **Rationale**: Backend switching should stay local to the app instead of leaking into rendering code.
- **Notes**: If the simulation extinguishes to an effectively zero-density field, this method immediately reseeds with a fresh randomized world so the viewer does not get stuck in an empty state.

### `ViewerApp::draw_controls`
-- **Does**: Renders the side-panel controls for execution, backend choice, food seeding, view selection, and Lenia parameters.
- **Interacts with**: `ViewMode` in `render.rs`, `LeniaParams` in `lenia-core`
- **Rationale**: The `Volume 3D` controls intentionally expose transfer-function settings instead of renderer-internal debug knobs so the CPU slice path and GPU volume path stay separate.

### `ViewerApp::apply_food_sources`
- **Does**: Periodically stamps 3D Gaussian food blobs into the world using fixed or randomized sources.
- **Interacts with**: `stamp_gaussian_blob_3d` in `lenia-core`

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
| Users of the viewer | Play/pause, step, backend switching, extinction auto-reseeding, periodic food seeding, 3D viewing, and view selection all function without external configuration | Removing controls or changing runtime behavior materially |

## Notes
- `Volume 3D` now uses a `wgpu` raymarch callback when the native app is running with the `eframe` `wgpu` backend enabled.
- Slice and MIP modes still go through the CPU texture path because they are the fastest debugging views for inspecting raw field data.
- Extinction is defined conservatively as `max_value <= 1e-6`, which tolerates tiny floating-point residue while still treating a visually dead field as dead.
