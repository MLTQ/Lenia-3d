# Lenia-3d

3D-first Lenia workspace focused on accurate simulation now and GPU-ready architecture next.

<img width="1388" height="909" alt="Lenia 3D viewer screenshot" src="https://github.com/user-attachments/assets/4242bcdb-1643-410b-9c49-dea674a150d0" />

## Current State

This repo is no longer just a bare simulation crate. It is a working 3D Lenia workspace with:

- `lenia-core`: a renderer-free simulation crate with reference and FFT stepping paths
- `viewer-egui`: a native `eframe` / `egui` app with a real `wgpu` volume raymarch viewport
- official 3D animal loading based on Chakazul's published `animals3D.json` data
- experimental multichannel / NDKC-style exploration wired into the viewer

The core math and loading path are tested. The viewer is usable for exploration. GPU compute simulation is not implemented yet.

## What You Can Do Today

- Run single-channel 3D Lenia with either the direct reference backend or the cached CPU FFT backend
- Load official 3D species and inspect them in `Volume 3D`, slice, or MIP modes
- Scale species loads to higher voxel resolution without automatically exploding the whole world size
- Resize the world destructively or preserve the current state with centered pad/crop
- Use periodic food seeding for exploratory closed/open-system behavior
- Inspect kernels with a center-slice heatmap and radial profile
- Switch into an experimental multichannel mode and edit rule parameters live

## Workspace Layout

- `crates/lenia-core`
  - dense `World3D` field type
  - reference periodic convolution
  - cached CPU FFT backend
  - official ND band-kernel family plus exploratory kernel families
  - RLE decoding and scale-aware seeding for official 3D species
  - experimental multichannel reference and FFT backends
- `crates/viewer-egui`
  - native desktop viewer
  - `wgpu` volume raymarch viewport
  - slice and MIP debugging views
  - play/pause/step controls
  - species loader, kernel editor, food controls, and multichannel controls
- `docs/roadmap.md`
  - staged plan toward faster backends and eventual GPU compute

## Current Capabilities

### Simulation

- Normalized 3D kernels
- Periodic boundary conditions
- Reference single-channel stepping for oracle behavior
- Cached FFT single-channel stepping for interactive volumes
- Experimental multichannel FFT stepping
- Multiple kernel families:
  - Gaussian shells
  - centered Gaussian stacks
  - official Lenia ND bands

### Viewer

- Orbitable 3D volume rendering through `wgpu`
- XY / XZ / YZ slice views
- XY / XZ / YZ max-intensity projections
- Species loading with scale control
- World resize with preserve-state mode
- Kernel preview panel
- Optional extinction auto-reseed
- Optional periodic food sources

## Known Limits

- Simulation is still CPU-side. The renderer uses `wgpu`, but the Lenia stepping backend is not GPU compute yet.
- Official 3D species support is real, but this is still a research/exploration tool rather than a strict parity port of the original Python UI.
- Multichannel mode is explicitly experimental. It is fast enough to explore, but the display path is still a max-over-channels composite.
- Volume rendering is intended for inspection, not yet for publication-grade transfer-function tuning.

## Run

```bash
cargo run -p viewer-egui
```

## Test And Format

```bash
cargo test
cargo fmt
```

## CI And Releases

GitHub Actions currently:

- build `viewer-egui` on every push
- target Linux x86_64 and macOS Apple Silicon
- upload build artifacts for each run
- publish tagged builds as GitHub Releases

## Credits

The shipped official 3D animals and source parameter data are credited to Chakazul's Lenia work and loaded from the published `animals3D.json` dataset. This repo does not claim authorship of those organisms.

Primary reference project:

- [Chakazul/Lenia](https://github.com/Chakazul/Lenia)

## Project Direction

The current architecture is deliberately split so the simulation contracts survive the eventual move to GPU compute:

- keep `lenia-core` independent of the UI
- keep reference paths available for parity testing
- keep the viewer useful as a debugging surface while faster backends are added

The next major technical milestone is GPU simulation, not a rewrite of the viewer.
