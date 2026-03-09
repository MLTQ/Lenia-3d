# lib.rs

## Purpose
Defines the public surface of the `lenia-core` crate. It keeps the crate root small and re-exports the stable building blocks needed by callers and future backends.

## Components

### `Real`
- **Does**: Sets the runtime scalar type for the reference implementation.
- **Interacts with**: All simulation modules in this crate
- **Rationale**: Keeping the scalar alias centralized makes later precision experiments less invasive.

### Backend re-exports
- **Does**: Exposes the shared backend trait plus the reference and FFT backend implementations.
- **Interacts with**: `backend.rs`, `reference.rs`, `fft.rs`
- **Rationale**: Callers should be able to swap stepping strategies without learning the internal module structure.

### Module re-exports
- **Does**: Exposes `World3D`, parameter types, kernel generation, and stepping APIs.
- **Interacts with**: `field.rs`, `params.rs`, `kernel.rs`, `growth.rs`, `seed.rs`, `simulator.rs`
- **Rationale**: Callers should not need to know the internal file layout to use the core.

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Future viewer crate | Stable access to `World3D`, `LeniaParams`, `KernelMode`, backend types, and seeding helpers | Renaming or removing re-exports |
| Future FFT/GPU backends | Shared `Real` alias and parameter model | Changing scalar alias semantics or parameter fields |

## Notes
- Keep this file thin. New functionality should usually land in a focused module with its own companion doc.
- The core now exposes a reusable 3D blob-stamping helper so viewers and preset systems can seed structure without reimplementing local world writes.
- The crate root now also re-exports `KernelMode` so UI code can switch kernel families without importing `params` directly.
