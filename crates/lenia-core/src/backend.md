# backend.rs

## Purpose
Defines the stable stepping interface shared by the reference, FFT, and future GPU backends. This file exists to keep callers insulated from backend-specific cache and execution details.

## Components

### `SimulationBackend`
- **Does**: Provides a minimal interface for one Lenia update step and backend identification.
- **Interacts with**: `ReferenceBackend` in `reference.rs`, `FftBackend` in `fft.rs`
- **Rationale**: A narrow contract makes it easier to swap acceleration strategies without changing simulation consumers.

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Future viewer crate | `step` advances a `World3D` for a given `LeniaParams` | Changing the method signature |
| `reference.rs`, `fft.rs` | Shared trait used for backend polymorphism | Removing trait methods or altering semantics |

## Notes
- Keep this trait intentionally small. Capability discovery and diagnostics should be separate from the stepping contract.
