# Roadmap

## Phase 1: Reference Core

- Establish a 3D volume abstraction and parameter model.
- Generate normalized 3D kernels from radial shell definitions.
- Implement a direct periodic convolution stepper.
- Lock behavior down with tests.

## Phase 2: CPU Acceleration

- Add kernel caching keyed by parameter set.
- Introduce FFT-based convolution for production-sized volumes.
- Benchmark reference vs FFT on representative grids.

Status:
- Kernel caching and the first CPU FFT backend are now implemented.
- Next step is profiling and tightening allocation behavior before the GPU port.

## Phase 3: Viewer

- Start with orthogonal slices and max-intensity projection.
- Add histogram and transfer-function tools once the simulation stabilizes.
- Delay full volume raymarching until the simulation contracts are stable.

Status:
- The `eframe` / `egui` viewer is implemented with play/pause/step controls, backend switching, periodic food seeding, species loading, multiple kernel families, a live kernel preview, a true orbitable `wgpu` raymarched viewport, and retained XY/XZ/YZ plus MIP inspection modes.
- The next viewer upgrade should focus on interaction quality: direct 3D painting tools, better transfer-function tooling, and hooking the experimental multi-channel path into the UI.

## Phase 4: GPU Backend

- Keep the public simulation interface backend-agnostic.
- Port convolution and growth evaluation to GPU compute.
- Preserve the reference backend as the correctness oracle for regression tests.
- Mirror the cache keying used by the CPU FFT backend so backend switching remains predictable.

Status:
- A separate experimental multi-channel reference engine now exists in `lenia-core` as the first architectural step toward NDKC-style systems.
- The next step is deciding how to expose multi-channel state and compositing in the viewer without breaking the current single-channel UX.
