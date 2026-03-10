use crate::{
    apply_growth_mapping,
    fft::{convolve_periodic_fft, kernel_spectrum_for},
    LeniaParams, MultiChannelParams, MultiChannelWorld, Real, World3D,
};
use ndarray::{Array3, Zip};
use rustfft::num_complex::Complex32;

#[derive(Default)]
pub struct MultiChannelFftBackend {
    cached: Vec<CachedSpectrum>,
}

#[derive(Clone)]
struct CachedSpectrum {
    params: LeniaParams,
    world_shape: (usize, usize, usize),
    kernel_spectrum: Array3<Complex32>,
}

impl MultiChannelFftBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_cached_kernel_for(
        &self,
        world_shape: (usize, usize, usize),
        params: &LeniaParams,
    ) -> bool {
        self.cached
            .iter()
            .any(|cached| cached.params == *params && cached.world_shape == world_shape)
    }

    pub fn step(
        &mut self,
        world: &MultiChannelWorld,
        params: &MultiChannelParams,
    ) -> MultiChannelWorld {
        assert_eq!(
            world.channel_count(),
            params.channel_count,
            "world and params channel counts must match"
        );

        let shape = world.shape();
        let mut accumulated_growth = vec![Array3::<Real>::zeros(shape); params.channel_count];

        for rule in &params.rules {
            assert!(
                rule.source_channel < params.channel_count
                    && rule.target_channel < params.channel_count,
                "rule channel index out of range"
            );

            let kernel_spectrum = self.spectrum_for(shape, &rule.params);
            let potential =
                convolve_periodic_fft(world.channel(rule.source_channel).view(), &kernel_spectrum);
            let growth = apply_growth_mapping(
                &potential,
                rule.params.mu,
                rule.params.safe_sigma(),
                rule.params.growth_function,
            );

            Zip::from(&mut accumulated_growth[rule.target_channel])
                .and(&growth)
                .for_each(|target_value, &growth_value| {
                    *target_value += growth_value * rule.weight * rule.params.safe_time_step();
                });
        }

        let mut next = MultiChannelWorld::zeros(params.channel_count, shape.0, shape.1, shape.2);
        for channel_index in 0..params.channel_count {
            let current = world.channel(channel_index).view();
            let growth = &accumulated_growth[channel_index];
            let mut output = Array3::<Real>::zeros(shape);
            Zip::from(&mut output).and(current).and(growth).for_each(
                |output_value, &current_value, &growth_value| {
                    *output_value = (current_value + growth_value).clamp(0.0, 1.0);
                },
            );
            *next.channel_mut(channel_index) = World3D::from_array(output);
        }

        next
    }

    fn spectrum_for(
        &mut self,
        world_shape: (usize, usize, usize),
        params: &LeniaParams,
    ) -> Array3<Complex32> {
        if let Some(cached) = self
            .cached
            .iter()
            .find(|cached| cached.params == *params && cached.world_shape == world_shape)
        {
            return cached.kernel_spectrum.clone();
        }

        let kernel_spectrum = kernel_spectrum_for(world_shape, params);
        self.cached.push(CachedSpectrum {
            params: params.clone(),
            world_shape,
            kernel_spectrum: kernel_spectrum.clone(),
        });
        kernel_spectrum
    }
}

#[cfg(test)]
mod tests {
    use super::MultiChannelFftBackend;
    use crate::{
        seed_ndkc_starter_world, step_multichannel_reference, KernelRule, MultiChannelParams,
    };
    use approx::assert_relative_eq;

    #[test]
    fn caches_kernel_spectra_for_repeated_rule_params() {
        let mut backend = MultiChannelFftBackend::new();
        let world_shape = (16, 16, 16);
        let base_params = crate::LeniaParams::default();
        let params = MultiChannelParams {
            channel_count: 2,
            rules: vec![
                KernelRule {
                    source_channel: 0,
                    target_channel: 0,
                    weight: 1.0,
                    params: base_params.clone(),
                },
                KernelRule {
                    source_channel: 1,
                    target_channel: 1,
                    weight: 0.7,
                    params: base_params.clone(),
                },
            ],
        };
        let world = crate::MultiChannelWorld::zeros(2, world_shape.0, world_shape.1, world_shape.2);

        let _ = backend.step(&world, &params);

        assert!(backend.has_cached_kernel_for(world_shape, &base_params));
        assert_eq!(backend.cached.len(), 1);
    }

    #[test]
    fn fft_backend_matches_reference_step() {
        let params = MultiChannelParams::ndkc_starter_preset();
        let world = seed_ndkc_starter_world((16, 16, 16), &params);
        let expected = step_multichannel_reference(&world, &params);
        let mut backend = MultiChannelFftBackend::new();
        let actual = backend.step(&world, &params);

        for channel_index in 0..expected.channel_count() {
            for (&expected_value, &actual_value) in expected
                .channel(channel_index)
                .view()
                .iter()
                .zip(actual.channel(channel_index).view().iter())
            {
                assert_relative_eq!(expected_value, actual_value, epsilon = 1.0e-4);
            }
        }
    }
}
