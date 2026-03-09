# simulator.rs

## Purpose
Contains the reference 3D stepping path. It is intentionally simple and exact rather than fast, so future FFT and GPU implementations have a correctness oracle.

## Components

### `convolve_periodic_reference`
- **Does**: Applies direct 3D convolution with periodic boundaries.
- **Interacts with**: `Kernel3D` from `kernel.rs`, `World3D` views from `field.rs`
- **Rationale**: Direct convolution is slow but easy to reason about and validate.

### `step_reference`
- **Does**: Runs one full Lenia update: kernel generation, convolution, growth mapping, and clamped state integration.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`, `apply_growth_mapping` in `growth.rs`

### `step_with_kernel`
- **Does**: Runs the reference update using a precomputed kernel view.
- **Interacts with**: `ReferenceBackend` in `reference.rs`
- **Rationale**: Allows backend-specific caching without duplicating the trusted stepping math.

### `integrate_from_potential`
- **Does**: Applies growth mapping and clamped state integration to a precomputed potential field.
- **Interacts with**: `step_reference` in this file, accelerated backends in other modules
- **Rationale**: Separates convolution from state integration so alternate backends can reuse the same update semantics.

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Future FFT backend tests | Numerical behavior stays authoritative for small volumes | Changing boundary conditions or update order |
| `reference.rs` | `step_with_kernel` reuses the same integration semantics as `step_reference` | Diverging formulas between the two APIs |
| Future viewer crate | `step_reference` returns a valid `World3D` in `[0, 1]` | Removing clamping or changing return type |

## Notes
- This file should remain the slow, trusted implementation even after acceleration lands elsewhere.
