# build-release.yml

## Purpose
Builds the native Lenia viewer on every push for the supported desktop targets and publishes release artifacts on tag pushes. This keeps normal branch pushes validating the shipping binary while reserving GitHub Releases for versioned tags.

## Components

### `build`
- **Does**: Builds `viewer-egui` in release mode for Linux x86_64 and macOS Apple Silicon, packages each binary as a `.tar.gz`, and uploads them as workflow artifacts.
- **Interacts with**: Cargo workspace at the repository root, GitHub Actions artifact storage

### `release`
- **Does**: Downloads the packaged artifacts from the matrix build and publishes them to a GitHub Release when the pushed ref is a tag.
- **Interacts with**: `softprops/action-gh-release`, uploaded artifacts from `build`

## Contracts

| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Contributors pushing branches | Every push produces build artifacts for both supported targets | Narrowing the trigger or removing a target |
| Maintainers pushing tags | Tag pushes create a GitHub Release containing both packaged binaries | Changing the tag-triggered release behavior |

## Notes
- Linux installs a small set of native packages required by the `eframe`/`wgpu` desktop stack on GitHub-hosted runners.
- The workflow publishes releases only for tags; plain branch pushes still build and upload artifacts but do not create GitHub Release objects.
