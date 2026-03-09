# reference.rs

## Purpose
Implements the direct CPU backend behind the shared backend trait. This file keeps the slow, exact stepping path usable as a regular backend while caching kernel generation between steps.

## Components

### `ReferenceBackend`
- **Does**: Executes one step using direct convolution and caches the most recent kernel by parameter set.
- **Interacts with**: `SimulationBackend` in `backend.rs`, `step_with_kernel` in `simulator.rs`
- **Rationale**: The reference path should stay convenient to use even after accelerated backends exist.

### `ReferenceBackend::has_cached_kernel_for`
- **Does**: Reports whether the backend currently holds a kernel for the supplied parameters.
- **Interacts with**: Tests in this file
- **Rationale**: Gives lightweight visibility into cache behavior without exposing the cache internals.

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Future backend benchmarks | Kernel generation is reused when parameters do not change | Removing caching behavior |
| Callers using `SimulationBackend` | `step` matches `step_reference` numerically | Diverging update behavior |

## Notes
- This backend should remain the correctness oracle, even if it becomes less important for interactive runtime use.
