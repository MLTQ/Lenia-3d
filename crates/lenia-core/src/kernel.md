# kernel.rs

## Purpose
Builds normalized 3D Lenia kernels from radial shell parameters. This file exists so kernel construction, normalization, and validation can evolve independently from the stepper.

## Components

### `Kernel3D`
- **Does**: Wraps the generated kernel weights and exposes read-only access.
- **Interacts with**: `convolve_periodic_reference` in `simulator.rs`

### `generate_kernel_3d`
- **Does**: Dispatches to the active 3D kernel family and returns a normalized discrete kernel.
- **Interacts with**: `LeniaParams` in `params.rs`, `step_reference` in `simulator.rs`
- **Rationale**: Kernel normalization must be centralized so all backends agree on total mass.

### `generate_shell_kernel_3d`
- **Does**: Expands shell centers/widths/weights into the current ring/shell kernel family.
- **Interacts with**: `KernelMode::GaussianShells` in `params.rs`

### `generate_centered_gaussian_kernel_3d`
- **Does**: Builds a legacy-style kernel by stacking center-aligned Gaussians using shell widths and weights.
- **Interacts with**: `KernelMode::CenteredGaussian` in `params.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `simulator.rs` | Kernel weights sum to `1.0` unless the kernel degenerates to a center impulse fallback | Changing normalization behavior |
| Future FFT backend | The discrete kernel layout matches the reference backend exactly for every `KernelMode` | Reordering axes or changing radius interpretation |

## Notes
- Shell centers are normalized by `radius_cells`, which keeps the same qualitative kernel family available at different grid resolutions.
- `CenteredGaussian` intentionally ignores shell centers so it behaves like the old 2D centered-Gaussian family while still reusing the same parameter container.
