# params.rs

## Purpose
Holds the public parameter model for 3D Lenia. This file defines the stable knobs shared by the reference backend today and the accelerated backends planned next.

## Components

### `GrowthFunction`
- **Does**: Enumerates the supported growth response curves.
- **Interacts with**: `map_growth_value` in `growth.rs`, future UI selection controls

### `KernelCore`
- **Does**: Enumerates the radial basis functions used by the official ND Lenia banded-kernel family.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`, official species presets in `species.rs`

### `KernelMode`
- **Does**: Selects between the exploratory shell family, the centered Gaussian stack, and the official ND Lenia banded kernel.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`, kernel controls in `viewer-egui`

### `KernelShell`
- **Does**: Describes one weighted radial shell within the normalized kernel radius.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`
- **Rationale**: Shells are easier to scale across resolutions than hard-coded per-cell kernel coefficients.

### `LeniaParams`
- **Does**: Groups kernel radius, growth parameters, shell definitions, and official band metadata into one simulation config.
- **Interacts with**: `step_reference` in `simulator.rs`, tests throughout the crate

### `LeniaParams::centered_gaussian_preset` / `LeniaParams::gaussian_rings_preset` / `LeniaParams::lenia_bands_preset`
- **Does**: Provides stable starting points for the three kernel families exposed by the viewer.
- **Interacts with**: Kernel-mode preset buttons in `viewer-egui`

### `LeniaParams::normalized_shells`
- **Does**: Repairs missing or invalid shell values before kernel generation.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`
- **Rationale**: Keeps defensive parameter cleanup out of the hot path call sites.

### `LeniaParams::normalized_bands`
- **Does**: Repairs missing or invalid band weights before official ND kernel generation.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `kernel.rs` | Shells are expressed in normalized radius space `[0, 1]` | Changing shell semantics |
| `kernel.rs` | `bands` and `kernel_core` define the official ND banded kernel when `kernel_mode == LeniaBands` | Changing official-kernel semantics |
| Future UI/viewer crate | `GrowthFunction`, `KernelCore`, `KernelMode`, and `LeniaParams` fields remain stable enough to expose controls | Renaming enum variants or removing fields |

## Notes
- `radius_cells` controls discrete kernel extent, while shell centers stay normalized so the same pattern family can be explored across resolutions.
- In `CenteredGaussian`, shell centers are ignored and shell widths/weights act like stacked center-aligned Gaussian components.
- `LeniaBands` exists specifically so the official 3D animals can use the original ND band list and kernel-core model instead of the repo's exploratory Gaussian families.
