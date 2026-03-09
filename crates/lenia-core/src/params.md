# params.rs

## Purpose
Holds the public parameter model for 3D Lenia. This file defines the stable knobs shared by the reference backend today and the accelerated backends planned next.

## Components

### `GrowthFunction`
- **Does**: Enumerates the supported growth response curves.
- **Interacts with**: `map_growth_value` in `growth.rs`, future UI selection controls

### `KernelShell`
- **Does**: Describes one weighted radial shell within the normalized kernel radius.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`
- **Rationale**: Shells are easier to scale across resolutions than hard-coded per-cell kernel coefficients.

### `LeniaParams`
- **Does**: Groups kernel radius, growth parameters, and shell definitions into one simulation config.
- **Interacts with**: `step_reference` in `simulator.rs`, tests throughout the crate

### `LeniaParams::normalized_shells`
- **Does**: Repairs missing or invalid shell values before kernel generation.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`
- **Rationale**: Keeps defensive parameter cleanup out of the hot path call sites.

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `kernel.rs` | Shells are expressed in normalized radius space `[0, 1]` | Changing shell semantics |
| Future UI/viewer crate | `GrowthFunction` names and `LeniaParams` fields remain stable enough to expose controls | Renaming enum variants or removing fields |

## Notes
- `radius_cells` controls discrete kernel extent, while shell centers stay normalized so the same pattern family can be explored across resolutions.
