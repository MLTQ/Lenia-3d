use crate::{KernelMode, LeniaParams, Real};
use ndarray::Array3;

#[derive(Clone, Debug, PartialEq)]
pub struct Kernel3D {
    weights: Array3<Real>,
}

impl Kernel3D {
    pub fn view(&self) -> ndarray::ArrayView3<'_, Real> {
        self.weights.view()
    }

    pub fn shape(&self) -> (usize, usize, usize) {
        self.weights.dim()
    }

    pub fn sum(&self) -> Real {
        self.weights.sum()
    }
}

pub fn generate_kernel_3d(params: &LeniaParams) -> Kernel3D {
    match params.kernel_mode {
        KernelMode::GaussianShells => generate_shell_kernel_3d(params),
        KernelMode::CenteredGaussian => generate_centered_gaussian_kernel_3d(params),
    }
}

fn generate_shell_kernel_3d(params: &LeniaParams) -> Kernel3D {
    let diameter = params.kernel_diameter();
    let center = params.radius_cells as Real;
    let shells = params.normalized_shells();
    let mut weights = Array3::zeros((diameter, diameter, diameter));

    for ((z, y, x), value) in weights.indexed_iter_mut() {
        let dz = z as Real - center;
        let dy = y as Real - center;
        let dx = x as Real - center;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let normalized_radius = if params.radius_cells == 0 {
            0.0
        } else {
            distance / params.radius_cells as Real
        };

        if normalized_radius > 1.0 {
            continue;
        }

        let shell_value = shells
            .iter()
            .map(|shell| {
                shell.weight * gaussian_shell(normalized_radius, shell.center, shell.width)
            })
            .sum::<Real>();

        *value = shell_value.max(0.0);
    }

    let total_weight = weights.sum();
    if total_weight > 0.0 {
        weights.mapv_inplace(|value| value / total_weight);
    } else {
        weights[(
            params.radius_cells,
            params.radius_cells,
            params.radius_cells,
        )] = 1.0;
    }

    Kernel3D { weights }
}

fn generate_centered_gaussian_kernel_3d(params: &LeniaParams) -> Kernel3D {
    let diameter = params.kernel_diameter();
    let center = params.radius_cells as Real;
    let shells = params.normalized_shells();
    let mut weights = Array3::zeros((diameter, diameter, diameter));

    for ((z, y, x), value) in weights.indexed_iter_mut() {
        let dz = z as Real - center;
        let dy = y as Real - center;
        let dx = x as Real - center;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let normalized_radius = if params.radius_cells == 0 {
            0.0
        } else {
            distance / params.radius_cells as Real
        };

        if normalized_radius > 1.0 {
            continue;
        }

        *value = shells
            .iter()
            .map(|shell| shell.weight * gaussian_shell(normalized_radius, 0.0, shell.width))
            .sum::<Real>()
            .max(0.0);
    }

    normalize_kernel(weights, params.radius_cells)
}

fn normalize_kernel(mut weights: Array3<Real>, radius_cells: usize) -> Kernel3D {
    let total_weight = weights.sum();
    if total_weight > 0.0 {
        weights.mapv_inplace(|value| value / total_weight);
    } else {
        weights[(radius_cells, radius_cells, radius_cells)] = 1.0;
    }

    Kernel3D { weights }
}

fn gaussian_shell(radius: Real, center: Real, width: Real) -> Real {
    let safe_width = width.max(1.0e-4);
    (-(radius - center).powi(2) / (2.0 * safe_width * safe_width)).exp()
}

#[cfg(test)]
mod tests {
    use super::generate_kernel_3d;
    use crate::{KernelMode, KernelShell, LeniaParams};
    use approx::assert_relative_eq;

    #[test]
    fn kernel_is_normalized() {
        let kernel = generate_kernel_3d(&LeniaParams::default());
        assert_relative_eq!(kernel.sum(), 1.0, epsilon = 1.0e-5);
    }

    #[test]
    fn kernel_shape_tracks_radius() {
        let params = LeniaParams {
            radius_cells: 4,
            ..LeniaParams::default()
        };
        let kernel = generate_kernel_3d(&params);
        assert_eq!(kernel.shape(), (9, 9, 9));
    }

    #[test]
    fn centered_gaussian_kernel_peaks_at_center() {
        let params = LeniaParams {
            kernel_mode: KernelMode::CenteredGaussian,
            shells: vec![
                KernelShell::new(0.8, 0.12, 1.0),
                KernelShell::new(0.5, 0.25, 0.5),
            ],
            ..LeniaParams::default()
        };
        let kernel = generate_kernel_3d(&params);
        let view = kernel.view();
        let c = params.radius_cells;
        assert!(view[(c, c, c)] > view[(c, c, c + 2)]);
    }
}
