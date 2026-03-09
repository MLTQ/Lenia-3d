# growth.rs

## Purpose
Implements the scalar and bulk growth-response functions for Lenia. This file isolates the non-linear update rule from kernel generation and convolution so the math can be tested independently.

## Components

### `map_growth_value`
- **Does**: Converts one convolution potential value into a growth response in `[-1, 1]`.
- **Interacts with**: `apply_growth_mapping` in this file, `step_reference` in `simulator.rs`

### `apply_growth_mapping`
- **Does**: Applies the selected growth function across a full 3D field.
- **Interacts with**: `convolve_periodic_reference` output in `simulator.rs`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `simulator.rs` | Growth outputs stay in a bounded range suitable for clamped state updates | Changing output range or function signatures |
| Future FFT/GPU backends | Identical scalar semantics to the reference path | Diverging formulas between backends |

## Notes
- Keep the scalar formula authoritative here. Accelerated backends should be validated against this implementation.
