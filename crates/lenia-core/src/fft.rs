use crate::{
    apply_growth_mapping, backend::SimulationBackend, generate_kernel_3d, Kernel3D, LeniaParams,
    Real, World3D,
};
use ndarray::{Array3, ArrayView3, Zip};
use rustfft::{num_complex::Complex32, FftPlanner};

#[derive(Default)]
pub struct FftBackend {
    cached: Option<CachedSpectrum>,
}

#[derive(Clone)]
struct CachedSpectrum {
    params: LeniaParams,
    world_shape: (usize, usize, usize),
    kernel_spectrum: Array3<Complex32>,
}

impl FftBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_cached_kernel_for(
        &self,
        world_shape: (usize, usize, usize),
        params: &LeniaParams,
    ) -> bool {
        self.cached
            .as_ref()
            .is_some_and(|cached| cached.params == *params && cached.world_shape == world_shape)
    }

    fn spectrum_for(
        &mut self,
        world_shape: (usize, usize, usize),
        params: &LeniaParams,
    ) -> Array3<Complex32> {
        if let Some(cached) = &self.cached {
            if cached.params == *params && cached.world_shape == world_shape {
                return cached.kernel_spectrum.clone();
            }
        }

        let kernel_volume = kernel_spectrum_for(world_shape, params);

        self.cached = Some(CachedSpectrum {
            params: params.clone(),
            world_shape,
            kernel_spectrum: kernel_volume.clone(),
        });

        kernel_volume
    }
}

impl SimulationBackend for FftBackend {
    fn name(&self) -> &'static str {
        "fft"
    }

    fn step(&mut self, world: &World3D, params: &LeniaParams) -> World3D {
        let world_shape = world.shape();
        let kernel_spectrum = self.spectrum_for(world_shape, params);
        let potential = convolve_periodic_fft(world.view(), &kernel_spectrum);
        integrate_from_potential(world, &potential, params)
    }
}

pub fn convolve_periodic_fft(
    input: ArrayView3<'_, Real>,
    kernel_spectrum: &Array3<Complex32>,
) -> Array3<Real> {
    let mut planner = FftPlanner::<Real>::new();
    let mut volume = input.mapv(|value| Complex32::new(value, 0.0));
    fft3_in_place(&mut volume, &mut planner, FftDirection::Forward);

    Zip::from(&mut volume)
        .and(kernel_spectrum)
        .for_each(|value, &kernel_value| {
            *value *= kernel_value;
        });

    fft3_in_place(&mut volume, &mut planner, FftDirection::Inverse);

    let scale = input.len() as Real;
    volume.mapv(|value| value.re / scale)
}

pub(crate) fn kernel_spectrum_for(
    world_shape: (usize, usize, usize),
    params: &LeniaParams,
) -> Array3<Complex32> {
    let kernel = generate_kernel_3d(params);
    let mut kernel_volume = embed_kernel_in_world(world_shape, &kernel);
    let mut planner = FftPlanner::<Real>::new();
    fft3_in_place(&mut kernel_volume, &mut planner, FftDirection::Forward);
    kernel_volume
}

fn integrate_from_potential(
    world: &World3D,
    potential: &Array3<Real>,
    params: &LeniaParams,
) -> World3D {
    let growth = apply_growth_mapping(
        potential,
        params.mu,
        params.safe_sigma(),
        params.growth_function,
    );

    if let Some(beta) = params.mace_beta {
        crate::simulator::apply_mace_update_3d(world, &growth, beta)
    } else {
        let mut next = Array3::zeros(world.shape());
        Zip::from(&mut next)
            .and(world.view())
            .and(&growth)
            .for_each(|next_value, &current_value, &growth_value| {
                *next_value =
                    (current_value + growth_value * params.safe_time_step()).clamp(0.0, 1.0);
            });
        World3D::from_array(next)
    }
}

fn embed_kernel_in_world(
    world_shape: (usize, usize, usize),
    kernel: &Kernel3D,
) -> Array3<Complex32> {
    let (world_depth, world_height, world_width) = world_shape;
    let (kernel_depth, kernel_height, kernel_width) = kernel.shape();
    let depth_pad = kernel_depth / 2;
    let height_pad = kernel_height / 2;
    let width_pad = kernel_width / 2;

    let mut embedded = Array3::from_elem(world_shape, Complex32::new(0.0, 0.0));
    let kernel_view = kernel.view();

    for kernel_z in 0..kernel_depth {
        for kernel_y in 0..kernel_height {
            for kernel_x in 0..kernel_width {
                let target_z = (kernel_z + world_depth - depth_pad % world_depth) % world_depth;
                let target_y = (kernel_y + world_height - height_pad % world_height) % world_height;
                let target_x = (kernel_x + world_width - width_pad % world_width) % world_width;
                embedded[(target_z, target_y, target_x)].re +=
                    kernel_view[(kernel_z, kernel_y, kernel_x)];
            }
        }
    }

    embedded
}

#[derive(Clone, Copy)]
enum FftDirection {
    Forward,
    Inverse,
}

fn fft3_in_place(
    volume: &mut Array3<Complex32>,
    planner: &mut FftPlanner<Real>,
    direction: FftDirection,
) {
    let (depth, height, width) = volume.dim();
    let fft_width = plan_fft(planner, width, direction);
    let fft_height = plan_fft(planner, height, direction);
    let fft_depth = plan_fft(planner, depth, direction);
    let mut scratch = vec![Complex32::new(0.0, 0.0); depth.max(height).max(width)];

    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                scratch[x] = volume[(z, y, x)];
            }
            fft_width.process(&mut scratch[..width]);
            for x in 0..width {
                volume[(z, y, x)] = scratch[x];
            }
        }
    }

    for z in 0..depth {
        for x in 0..width {
            for y in 0..height {
                scratch[y] = volume[(z, y, x)];
            }
            fft_height.process(&mut scratch[..height]);
            for y in 0..height {
                volume[(z, y, x)] = scratch[y];
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            for z in 0..depth {
                scratch[z] = volume[(z, y, x)];
            }
            fft_depth.process(&mut scratch[..depth]);
            for z in 0..depth {
                volume[(z, y, x)] = scratch[z];
            }
        }
    }
}

fn plan_fft(
    planner: &mut FftPlanner<Real>,
    len: usize,
    direction: FftDirection,
) -> std::sync::Arc<dyn rustfft::Fft<Real>> {
    match direction {
        FftDirection::Forward => planner.plan_fft_forward(len),
        FftDirection::Inverse => planner.plan_fft_inverse(len),
    }
}

#[cfg(test)]
mod tests {
    use super::{convolve_periodic_fft, embed_kernel_in_world, FftBackend};
    use crate::{
        convolve_periodic_reference, generate_kernel_3d, LeniaParams, SimulationBackend, World3D,
    };
    use approx::assert_relative_eq;
    use ndarray::Array3;
    use rustfft::FftPlanner;

    fn test_world() -> World3D {
        World3D::from_array(Array3::from_shape_fn((4, 4, 4), |(z, y, x)| {
            ((z * 7 + y * 5 + x * 3) as f32 % 11.0) / 10.0
        }))
    }

    #[test]
    fn fft_convolution_matches_reference() {
        let params = LeniaParams {
            radius_cells: 2,
            ..LeniaParams::default()
        };
        let world = test_world();
        let kernel = generate_kernel_3d(&params);
        let kernel_spectrum = {
            let mut embedded = embed_kernel_in_world(world.shape(), &kernel);
            let mut planner = FftPlanner::<f32>::new();
            super::fft3_in_place(&mut embedded, &mut planner, super::FftDirection::Forward);
            embedded
        };

        let fft = convolve_periodic_fft(world.view(), &kernel_spectrum);
        let reference = convolve_periodic_reference(world.view(), kernel.view());

        for (fft_value, reference_value) in fft.iter().zip(reference.iter()) {
            assert_relative_eq!(fft_value, reference_value, epsilon = 1.0e-4);
        }
    }

    #[test]
    fn fft_backend_matches_reference_step() {
        let params = LeniaParams {
            radius_cells: 2,
            ..LeniaParams::default()
        };
        let world = test_world();
        let mut fft_backend = FftBackend::new();
        let mut reference_backend = crate::ReferenceBackend::new();

        let fft_next = fft_backend.step(&world, &params);
        let reference_next = reference_backend.step(&world, &params);

        for (fft_value, reference_value) in fft_next.view().iter().zip(reference_next.view().iter())
        {
            assert_relative_eq!(fft_value, reference_value, epsilon = 1.0e-4);
        }
    }

    #[test]
    fn caches_kernel_spectrum_for_identical_shape_and_params() {
        let mut backend = FftBackend::new();
        let world = test_world();
        let params = LeniaParams {
            radius_cells: 2,
            ..LeniaParams::default()
        };

        assert!(!backend.has_cached_kernel_for(world.shape(), &params));
        let _ = backend.step(&world, &params);
        assert!(backend.has_cached_kernel_for(world.shape(), &params));
    }
}
