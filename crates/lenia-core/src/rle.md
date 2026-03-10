# rle.rs

## Purpose
Decodes the official ND Lenia run-length encoding used by the published species files and embeds decoded seeds into a `World3D`. This file keeps Chakazul-compatible seed parsing separate from the simulation and viewer code.

## Components

### `decode_lenia_rle_3d`
- **Does**: Parses the official 3D Lenia RLE stream into a dense `Array3<Real>`.
- **Interacts with**: `seeded_world_for_preset` in `species.rs`

### `centered_world_from_rle`
- **Does**: Centers a decoded 3D seed into a target `World3D` with periodic wrapping.
- **Interacts with**: `World3D` in `field.rs`, `single_species_presets` in `species.rs`

### `centered_scaled_world_from_rle`
- **Does**: Decodes a 3D seed, scales it by integer nearest-neighbor voxel replication, and then centers it into a target `World3D`.
- **Interacts with**: `seeded_world_for_preset_scaled` in `species.rs`

### `decode_lenia_value`
- **Does**: Converts Lenia's compact cell tokens into normalized density values.
- **Interacts with**: `decode_lenia_rle_3d`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `species.rs` | Official `cells` strings from `animals3D.json` decode into the same scalar range as the Python implementation, with optional integer lattice upscaling for higher-resolution loads | Changing token mapping or delimiter semantics |
| Future import tooling | Seeds are centered into worlds the same way every time | Changing centering or wrap behavior |

## Notes
- This implementation is intentionally 3D-specific because the current viewer and species library are 3D-only.
- The parser mirrors the original ND delimiter semantics for 3D: `$` ends a row and `%` ends a slice.
- Scaling uses simple voxel replication on purpose so official seeds can be enlarged deterministically before any later interpolation or search tooling exists.
