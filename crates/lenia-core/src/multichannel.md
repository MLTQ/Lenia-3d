# multichannel.rs

## Purpose
Provides an experimental multi-channel Lenia reference engine. This file starts the architectural move toward NDKC-style systems without replacing the existing single-channel backend contracts used by the viewer today.

## Components

### `MultiChannelWorld`
- **Does**: Stores multiple `World3D` channels with a shared shape.
- **Interacts with**: `step_multichannel_reference`, future multi-channel viewers/backends

### `KernelRule`
- **Does**: Defines one source-channel to target-channel interaction with its own kernel/growth parameters and coupling weight.
- **Interacts with**: `MultiChannelParams`, `step_multichannel_reference`

### `MultiChannelParams`
- **Does**: Groups the channel count and interaction rules for experimental multi-channel Lenia.
- **Interacts with**: `step_multichannel_reference`

### `MultiChannelParams::ndkc_starter_preset`
- **Does**: Supplies a small three-channel interaction graph to seed experimentation.
- **Interacts with**: Tests and future UI wiring

### `seed_ndkc_starter_world`
- **Does**: Builds a small structured three-channel seed volume that matches `ndkc_starter_preset`.
- **Interacts with**: Viewer startup and reseeding flows, `stamp_gaussian_blob_3d` in `seed.rs`

### `step_multichannel_reference`
- **Does**: Executes the reference multi-channel update by convolving each rule, mapping growth, accumulating into target channels, and clamping the result.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`, `apply_growth_mapping` in `growth.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Experimental viewers | `seed_ndkc_starter_world` returns a populated world that matches the declared channel count | Changing seeding semantics or channel layout |
| Future experimental viewers | Rule semantics remain source-channel -> target-channel with per-rule weights | Changing coupling direction or weighting semantics |
| Future accelerated multi-channel backends | This reference implementation remains the oracle for numerical parity | Changing update ordering or clamping behavior |

## Notes
- This is intentionally separate from the current `SimulationBackend` trait because the shipping viewer still consumes a single `World3D`.
- The starter preset is exploratory and NDKC-inspired, not a claim of exact parity with the official multi-kernel/multi-channel implementation.
- The starter world intentionally uses overlapping Gaussian blobs instead of pure noise so the experimental viewer has a visually coherent seed to evolve.
- The starter preset still uses the exploratory Gaussian families, so its `LeniaParams` fill the new official-kernel fields with defaults rather than opting into `LeniaBands`.
