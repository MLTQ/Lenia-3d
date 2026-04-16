use crate::{
    apply_growth_mapping, convolve_periodic_reference, generate_kernel_3d, stamp_gaussian_blob_3d,
    KernelMode, KernelShell, LeniaParams, Real, World3D,
};
use ndarray::{Array3, Zip};

#[derive(Clone, Debug, PartialEq)]
pub struct MultiChannelWorld {
    channels: Vec<World3D>,
}

impl MultiChannelWorld {
    pub fn zeros(channel_count: usize, depth: usize, height: usize, width: usize) -> Self {
        assert!(channel_count > 0, "channel count must be positive");
        Self {
            channels: (0..channel_count)
                .map(|_| World3D::zeros(depth, height, width))
                .collect(),
        }
    }

    pub fn shape(&self) -> (usize, usize, usize) {
        self.channels[0].shape()
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn channel(&self, index: usize) -> &World3D {
        &self.channels[index]
    }

    pub fn channel_mut(&mut self, index: usize) -> &mut World3D {
        &mut self.channels[index]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KernelRule {
    pub source_channel: usize,
    pub target_channel: usize,
    pub weight: Real,
    pub params: LeniaParams,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiChannelParams {
    pub channel_count: usize,
    pub rules: Vec<KernelRule>,
}

impl MultiChannelParams {
    pub fn ndkc_starter_preset() -> Self {
        Self {
            channel_count: 3,
            rules: vec![
                KernelRule {
                    source_channel: 0,
                    target_channel: 0,
                    weight: 1.0,
                    params: LeniaParams {
                        kernel_mode: KernelMode::GaussianShells,
                        kernel_core: crate::KernelCore::Exponential,
                        radius_cells: 8,
                        mu: 0.29,
                        sigma: 0.09,
                        time_step: 0.06,
                        growth_function: crate::GrowthFunction::Exponential,
                        shells: vec![
                            KernelShell::new(0.20, 0.08, 1.0),
                            KernelShell::new(0.47, 0.07, 0.55),
                        ],
                        bands: vec![1.0],
                        mace_beta: None,
                    },
                },
                KernelRule {
                    source_channel: 0,
                    target_channel: 1,
                    weight: 0.45,
                    params: LeniaParams {
                        kernel_mode: KernelMode::CenteredGaussian,
                        kernel_core: crate::KernelCore::Exponential,
                        radius_cells: 7,
                        mu: 0.31,
                        sigma: 0.12,
                        time_step: 0.05,
                        growth_function: crate::GrowthFunction::Exponential,
                        shells: vec![
                            KernelShell::new(0.0, 0.08, 1.0),
                            KernelShell::new(0.0, 0.18, 0.70),
                        ],
                        bands: vec![1.0],
                        mace_beta: None,
                    },
                },
                KernelRule {
                    source_channel: 1,
                    target_channel: 2,
                    weight: 0.35,
                    params: LeniaParams {
                        kernel_mode: KernelMode::GaussianShells,
                        kernel_core: crate::KernelCore::Exponential,
                        radius_cells: 6,
                        mu: 0.27,
                        sigma: 0.08,
                        time_step: 0.05,
                        growth_function: crate::GrowthFunction::Exponential,
                        shells: vec![
                            KernelShell::new(0.18, 0.07, 1.0),
                            KernelShell::new(0.58, 0.08, 0.45),
                        ],
                        bands: vec![1.0],
                        mace_beta: None,
                    },
                },
                KernelRule {
                    source_channel: 2,
                    target_channel: 0,
                    weight: -0.20,
                    params: LeniaParams {
                        kernel_mode: KernelMode::CenteredGaussian,
                        kernel_core: crate::KernelCore::Exponential,
                        radius_cells: 5,
                        mu: 0.34,
                        sigma: 0.11,
                        time_step: 0.04,
                        growth_function: crate::GrowthFunction::Exponential,
                        shells: vec![KernelShell::new(0.0, 0.12, 1.0)],
                        bands: vec![1.0],
                        mace_beta: None,
                    },
                },
            ],
        }
    }
}

pub fn seed_ndkc_starter_world(
    world_shape: (usize, usize, usize),
    params: &MultiChannelParams,
) -> MultiChannelWorld {
    let (depth, height, width) = world_shape;
    let mut world = MultiChannelWorld::zeros(params.channel_count, depth, height, width);
    let center = (
        depth.saturating_sub(1) / 2,
        height.saturating_sub(1) / 2,
        width.saturating_sub(1) / 2,
    );

    if params.channel_count >= 1 {
        stamp_gaussian_blob_3d(
            world.channel_mut(0),
            center.0,
            center.1,
            center.2,
            13,
            0.85,
            0.0,
            0.22,
        );
        stamp_gaussian_blob_3d(
            world.channel_mut(0),
            center.0,
            center.1,
            (center.2 + 7).min(width.saturating_sub(1)),
            9,
            0.35,
            0.0,
            0.24,
        );
    }

    if params.channel_count >= 2 {
        stamp_gaussian_blob_3d(
            world.channel_mut(1),
            center.0.saturating_add(4).min(depth.saturating_sub(1)),
            center.1.saturating_sub(5),
            center.2.saturating_add(3).min(width.saturating_sub(1)),
            9,
            0.55,
            0.0,
            0.25,
        );
    }

    if params.channel_count >= 3 {
        stamp_gaussian_blob_3d(
            world.channel_mut(2),
            center.0.saturating_sub(4),
            center.1.saturating_add(4).min(height.saturating_sub(1)),
            center.2.saturating_sub(3),
            9,
            0.50,
            0.0,
            0.25,
        );
    }

    world
}

pub fn step_multichannel_reference(
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

        let kernel = generate_kernel_3d(&rule.params);
        let potential =
            convolve_periodic_reference(world.channel(rule.source_channel).view(), kernel.view());
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
                let next_value: Real = current_value + growth_value;
                *output_value = next_value.max(0.0_f32).min(1.0_f32);
            },
        );
        *next.channel_mut(channel_index) = World3D::from_array(output);
    }

    next
}

#[cfg(test)]
mod tests {
    use super::{
        seed_ndkc_starter_world, step_multichannel_reference, MultiChannelParams, MultiChannelWorld,
    };

    #[test]
    fn ndkc_starter_preset_matches_world_channel_count() {
        let params = MultiChannelParams::ndkc_starter_preset();
        let world = MultiChannelWorld::zeros(params.channel_count, 16, 16, 16);
        let next = step_multichannel_reference(&world, &params);
        assert_eq!(next.channel_count(), params.channel_count);
    }

    #[test]
    fn multichannel_step_clamps_channels() {
        let params = MultiChannelParams::ndkc_starter_preset();
        let mut world = MultiChannelWorld::zeros(params.channel_count, 16, 16, 16);
        world.channel_mut(0).set(8, 8, 8, 1.0);
        let next = step_multichannel_reference(&world, &params);
        for channel in 0..next.channel_count() {
            assert!(next
                .channel(channel)
                .view()
                .iter()
                .all(|value| (0.0..=1.0).contains(value)));
        }
    }

    #[test]
    fn starter_world_seeds_population() {
        let params = MultiChannelParams::ndkc_starter_preset();
        let world = seed_ndkc_starter_world((32, 32, 32), &params);
        assert!(world.channel(0).mean() > 0.0);
    }
}
