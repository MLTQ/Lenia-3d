use crate::{apply_growth_mapping, generate_kernel_3d, LeniaParams, Real, World3D};
use ndarray::{Array3, ArrayView3, Zip};

pub fn convolve_periodic_reference(
    input: ArrayView3<'_, Real>,
    kernel: ArrayView3<'_, Real>,
) -> Array3<Real> {
    let (depth, height, width) = input.dim();
    let (kernel_depth, kernel_height, kernel_width) = kernel.dim();
    assert!(
        kernel_depth % 2 == 1 && kernel_height % 2 == 1 && kernel_width % 2 == 1,
        "kernel dimensions must be odd"
    );

    let depth_pad = (kernel_depth / 2) as isize;
    let height_pad = (kernel_height / 2) as isize;
    let width_pad = (kernel_width / 2) as isize;
    let mut output = Array3::zeros((depth, height, width));

    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0;
                for kernel_z in 0..kernel_depth {
                    for kernel_y in 0..kernel_height {
                        for kernel_x in 0..kernel_width {
                            let source_z =
                                wrap_index(z as isize + kernel_z as isize - depth_pad, depth);
                            let source_y =
                                wrap_index(y as isize + kernel_y as isize - height_pad, height);
                            let source_x =
                                wrap_index(x as isize + kernel_x as isize - width_pad, width);
                            sum += input[(source_z, source_y, source_x)]
                                * kernel[(kernel_z, kernel_y, kernel_x)];
                        }
                    }
                }
                output[(z, y, x)] = sum;
            }
        }
    }

    output
}

pub fn step_reference(world: &World3D, params: &LeniaParams) -> World3D {
    let kernel = generate_kernel_3d(params);
    step_with_kernel(world, params, kernel.view())
}

pub fn step_with_kernel(
    world: &World3D,
    params: &LeniaParams,
    kernel: ArrayView3<'_, Real>,
) -> World3D {
    let potential = convolve_periodic_reference(world.view(), kernel);
    integrate_from_potential(world, &potential, params)
}

pub fn integrate_from_potential(
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
    let mut next = Array3::zeros(world.shape());
    Zip::from(&mut next)
        .and(world.view())
        .and(&growth)
        .for_each(|next_value, &current_value, &growth_value| {
            *next_value = (current_value + growth_value * params.safe_time_step()).clamp(0.0, 1.0);
        });

    World3D::from_array(next)
}

fn wrap_index(index: isize, size: usize) -> usize {
    index.rem_euclid(size as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::{convolve_periodic_reference, step_reference};
    use crate::{LeniaParams, World3D};
    use ndarray::Array3;

    #[test]
    fn periodic_convolution_wraps_across_volume_edges() {
        let mut input = Array3::zeros((3, 3, 3));
        input[(0, 0, 0)] = 1.0;

        let kernel = Array3::from_elem((3, 3, 3), 1.0);
        let output = convolve_periodic_reference(input.view(), kernel.view());

        assert!(output.iter().all(|value| *value > 0.0));
    }

    #[test]
    fn step_reference_clamps_world_state() {
        let mut world = World3D::zeros(5, 5, 5);
        world.set(2, 2, 2, 1.0);

        let next = step_reference(&world, &LeniaParams::default());
        assert!(next.view().iter().all(|value| (0.0..=1.0).contains(value)));
    }
}
