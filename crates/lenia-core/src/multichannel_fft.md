# multichannel_fft.rs

## Purpose
Provides the first accelerated multichannel Lenia backend using FFT-based circular convolution. This file keeps the multichannel reference oracle intact while giving the viewer a practical path for interactive multi-rule stepping.

## Components

### `MultiChannelFftBackend`
- **Does**: Executes multichannel Lenia steps using cached kernel spectra per world shape and rule parameters.
- **Interacts with**: `MultiChannelWorld` and `MultiChannelParams` in `multichannel.rs`, FFT helpers in `fft.rs`

### `MultiChannelFftBackend::step`
- **Does**: Convolves each rule in frequency space, applies growth, accumulates into target channels, and clamps the next world.
- **Interacts with**: `convolve_periodic_fft` and `kernel_spectrum_for` in `fft.rs`, `apply_growth_mapping` in `growth.rs`
- **Rationale**: Multichannel mode needs the same algorithmic upgrade that made the single-channel viewer interactive.

### `MultiChannelFftBackend::spectrum_for`
- **Does**: Reuses previously computed kernel spectra when multiple rules share parameters or when the world shape is unchanged between frames.
- **Interacts with**: Internal cache entries keyed by `LeniaParams` and world shape

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `viewer-egui` multichannel mode | Interactive stepping with the same rule semantics as `step_multichannel_reference` | Changing accumulation, clamping, or cache invalidation semantics |
| Backend parity tests | FFT stepping stays numerically close to the multichannel reference stepper on small worlds | Changing FFT normalization or kernel embedding |

## Notes
- This backend caches by `LeniaParams` and world shape, not by full rule identity, because source/target channels and coupling weights do not affect the kernel spectrum.
- The reference backend in `multichannel.rs` remains the numerical oracle for tests and debugging.
