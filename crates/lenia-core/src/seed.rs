use crate::{Real, World3D};

pub fn stamp_gaussian_blob_3d(
    world: &mut World3D,
    center_z: usize,
    center_y: usize,
    center_x: usize,
    size: usize,
    amplitude: Real,
    mu: Real,
    sigma: Real,
) {
    let blob_size = size.max(1);
    let half = (blob_size as isize) / 2;
    let sigma = sigma.max(1.0e-4);
    let (depth, height, width) = world.shape();
    let depth = depth as isize;
    let height = height as isize;
    let width = width as isize;

    for local_z in 0..blob_size {
        for local_y in 0..blob_size {
            for local_x in 0..blob_size {
                let z = center_z as isize + local_z as isize - half;
                let y = center_y as isize + local_y as isize - half;
                let x = center_x as isize + local_x as isize - half;

                if z < 0 || y < 0 || x < 0 || z >= depth || y >= height || x >= width {
                    continue;
                }

                let normalized_z = normalized_axis(local_z, blob_size);
                let normalized_y = normalized_axis(local_y, blob_size);
                let normalized_x = normalized_axis(local_x, blob_size);
                let distance = (normalized_x * normalized_x
                    + normalized_y * normalized_y
                    + normalized_z * normalized_z)
                    .sqrt();
                let amount = amplitude * (-((distance - mu).powi(2)) / (2.0 * sigma * sigma)).exp();

                let current = world.get(z as usize, y as usize, x as usize);
                world.set(z as usize, y as usize, x as usize, current + amount);
            }
        }
    }
}

fn normalized_axis(index: usize, size: usize) -> Real {
    if size <= 1 {
        0.0
    } else {
        2.0 * index as Real / (size - 1) as Real - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::stamp_gaussian_blob_3d;
    use crate::World3D;

    #[test]
    fn stamp_gaussian_blob_raises_local_density() {
        let mut world = World3D::zeros(9, 9, 9);
        stamp_gaussian_blob_3d(&mut world, 4, 4, 4, 5, 0.4, 0.0, 0.35);

        assert!(world.get(4, 4, 4) > 0.0);
    }

    #[test]
    fn stamp_gaussian_blob_clamps_output() {
        let mut world = World3D::zeros(5, 5, 5);
        stamp_gaussian_blob_3d(&mut world, 2, 2, 2, 5, 3.0, 0.0, 0.25);

        assert!(world.view().iter().all(|value| (0.0..=1.0).contains(value)));
    }
}
