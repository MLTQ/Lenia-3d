# species.rs

## Purpose
Defines the shipped official 3D species presets for the Lenia viewer. This file keeps named parameter bundles and source RLE seeds out of the app code so canonical-species curation can evolve independently from rendering and simulation backends.

## Components

### `SingleSpeciesPreset`
- **Does**: Bundles a display name, provenance note, preferred runtime settings, official code, Lenia parameters, and the original RLE seed.
- **Interacts with**: Viewer controls in `viewer-egui`, `seeded_world_for_preset`

### `single_species_presets`
- **Does**: Returns the curated set of official single-channel 3D species currently shipped with the project.
- **Interacts with**: `ViewerApp` in `viewer-egui`

### `seeded_world_for_preset`
- **Does**: Creates a fresh world and centers the preset's official RLE seed into it.
- **Interacts with**: `centered_world_from_rle` in `rle.rs`, `World3D` in `field.rs`

### `scaled_params_for_preset`
- **Does**: Produces a species-specific parameter set whose `radius_cells` has been scaled by an integer factor.
- **Interacts with**: `ViewerApp::apply_selected_species` in `viewer-egui`

### `scaled_seed_shape_for_preset`
- **Does**: Reports the decoded seed dimensions after integer voxel upscaling so callers can size a world around the scaled organism without creating an unnecessarily huge box.
- **Interacts with**: `ViewerApp::apply_selected_species` in `viewer-egui`

### `seeded_world_for_preset_scaled`
- **Does**: Creates a fresh world and centers an integer-upscaled copy of the preset's official RLE seed into it.
- **Interacts with**: `centered_scaled_world_from_rle` in `rle.rs`, `ViewerApp::apply_selected_species` in `viewer-egui`

### `official_lenia_params`
- **Does**: Converts the official `R`, `b`, `m`, `s`, `kn`, and `gn` metadata into `LeniaParams`.
- **Interacts with**: `KernelMode::LeniaBands`, `KernelCore`, and `GrowthFunction` in `params.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `viewer-egui` | Presets include parameters, preferred world size, and a decodable seed source, plus helpers for scale-aware loading | Removing fields or changing world-seeding semantics |
| Future preset tooling | `source_note` and `official_code` clearly preserve provenance back to the official species data | Removing provenance metadata |

## Notes
- These presets now come from Chakazul's `animals3D.json` instead of inferred Gaussian blob seeds.
- The current shipped set now spans multiple official 3D families, including Guttidae, Sphaeridae, Disphaerome, and Ovome examples.
- Presets still include a preferred world size so loading a species also restores the scale it was screened against instead of inheriting an arbitrary prior viewer volume.
- Scale-aware loading deliberately changes both the seed lattice and `radius_cells`; increasing the world size alone is not treated as a substitute for higher organism resolution, and the viewer now uses the scaled seed dimensions to recommend a box size instead of scaling the entire world edge one-for-one.
- The shipped values are screened with a short FFT locality probe so the viewer does not ship obviously domain-filling species loads by default.
