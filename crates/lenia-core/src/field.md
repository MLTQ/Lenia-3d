# field.rs

## Purpose
Wraps the raw `ndarray::Array3` used for simulation state. This file exists to keep volume-specific invariants and convenience operations in one place instead of leaking raw arrays across the whole crate.

## Components

### `World3D`
- **Does**: Owns a dense `(depth, height, width)` Lenia volume.
- **Interacts with**: `step_reference` in `simulator.rs`, kernel tests in `kernel.rs`
- **Rationale**: A dedicated type gives later backends one stable state contract.

### `World3D::zeros`, `World3D::random`, `World3D::from_array`
- **Does**: Constructs valid worlds with positive dimensions.
- **Interacts with**: Callers in tests and future seeding tools

### `World3D::fill`, `World3D::set`
- **Does**: Applies clamped writes in the Lenia state range `[0, 1]`.
- **Interacts with**: Tests and future painting/seeding layers

### `World3D::view`, `World3D::view_mut`, `World3D::into_array`
- **Does**: Exposes the underlying array when math modules need direct ndarray access.
- **Interacts with**: `convolve_periodic_reference` in `simulator.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `simulator.rs` | Dense 3D views with positive dimensions | Changing storage shape or removing view accessors |
| Future viewer crate | Read access by `(z, y, x)` and shape query | Renaming or removing `get`/`shape` |

## Notes
- Keep mutation rules centralized here so later GPU-backed state objects can preserve the same high-level API.
