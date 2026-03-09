# fft.rs

## Purpose
Implements the first accelerated CPU backend using FFT-based circular convolution. This file exists to validate the scalable backend shape before the project moves the same ideas onto GPU compute.

## Components

### `FftBackend`
- **Does**: Executes one Lenia step using cached kernel spectra keyed by world shape and parameters.
- **Interacts with**: `SimulationBackend` in `backend.rs`, `generate_kernel_3d` in `kernel.rs`

### `convolve_periodic_fft`
- **Does**: Performs 3D circular convolution in the frequency domain.
- **Interacts with**: `fft3_in_place` in this file, `convolve_periodic_reference` tests in `simulator.rs`
- **Rationale**: This is the first practical path toward larger volumes without changing the core math.

### `embed_kernel_in_world`
- **Does**: Places the centered discrete kernel into a world-sized circular-convolution volume.
- **Interacts with**: `generate_kernel_3d` in `kernel.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Backend equivalence tests | FFT convolution matches the reference convolution on small volumes | Changing kernel embedding or FFT normalization |
| Future GPU backend | World-shape-aware kernel caching semantics | Removing shape-sensitive cache behavior |

## Notes
- The implementation favors clarity over micro-optimization. It is a CPU scaling bridge, not the final performance endpoint.
