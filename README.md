# Lenia-3d

3D-first Lenia workspace aimed at accurate reference simulation now and scalable FFT/GPU backends next.

## Current Status

The repo now starts with a documented `lenia-core` crate:

- 3D dense volume type
- normalized 3D radial kernel generation
- reference periodic convolution stepper
- backend trait with cached reference and CPU FFT implementations
- multiple 3D kernel families, including the original centered-Gaussian-style mode and the shell/ring mode
- reusable 3D Gaussian seeding helper for presets and food sources
- `eframe` / `egui` viewer with a real orbitable 3D viewport using a `wgpu` volume raymarcher, periodic food seeding, kernel presets, a live kernel preview, plus slice and MIP inspection modes
- tests that define correctness for future accelerated backends

This first milestone is intentionally CPU-reference work. It gives the project a trustworthy baseline before introducing FFT and GPU complexity.
The second milestone adds the first scalable backend shape on CPU so the eventual GPU port can inherit the same contracts.
The viewer now starts bridging into the eventual GPU architecture by rendering the simulation volume through a dedicated `wgpu` callback inside the egui app.

## Workspace Layout

- `crates/lenia-core`: pure simulation crate with no renderer or windowing dependency
- `crates/viewer-egui`: runnable native viewer for stepping and inspecting 3D volumes
- `docs/roadmap.md`: staged architecture plan toward FFT and GPU execution

## Why This Shape

High-performance 3D Lenia needs the simulation core to stay independent from rendering. The viewer and the eventual GPU backend should be replaceable without changing the math contracts.
The current core now exposes a shared backend interface, which makes the direct oracle and FFT path swappable behind one stepping API.
The current viewer now defaults to a true 3D viewport driven by a volume shader, while retaining slice/MIP modes for debugging and inspection.
It also now exposes the kernel as a first-class object in the UI again, with a center-slice heatmap and radial profile similar to the older 2D tool.

## Commands

```bash
cargo run -p viewer-egui
cargo test
cargo fmt
```

## CI And Releases

- Every GitHub push builds `viewer-egui` for Linux x86_64 and macOS Apple Silicon and uploads the packaged binaries as workflow artifacts.
- Pushing a tag also publishes those artifacts as a GitHub Release.

## Beads Note

`beads` is initialized for this repository. Ongoing architecture work should continue to open, update, close, and sync beads so the project history survives across sessions.
