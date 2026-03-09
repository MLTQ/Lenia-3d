# seed.rs

## Purpose
Provides reusable world-seeding helpers for 3D Lenia volumes. This file keeps blob stamping and related initialization logic out of the viewer and backend code.

## Components

### `stamp_gaussian_blob_3d`
- **Does**: Adds a spherical Gaussian blob into a `World3D`, clamping the result into the valid life range.
- **Interacts with**: `World3D` in `field.rs`, periodic food controls in `viewer-egui`
- **Rationale**: Seeding should be reusable by presets, food systems, and future experiment tooling.

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| `viewer-egui` | Blob stamping remains bounded to the world and clamps cell values to `[0, 1]` | Changing coordinate order or clamping semantics |
| Future preset systems | Blob shape is spherical in normalized local coordinates | Changing blob geometry |

## Notes
- The blob uses normalized local coordinates so its qualitative shape stays stable as the discrete blob size changes.
