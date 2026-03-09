# main.rs

## Purpose
Defines the binary entry point for the `viewer-egui` crate. This file exists only to configure the native window and launch the egui application state.

## Components

### `main`
- **Does**: Creates the native window and starts `ViewerApp`.
- **Interacts with**: `ViewerApp` in `app.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `cargo run -p viewer-egui` | Launches a native viewer window successfully | Changing the binary target or startup signature |

## Notes
- Keep startup logic minimal; application behavior belongs in `app.rs`.
