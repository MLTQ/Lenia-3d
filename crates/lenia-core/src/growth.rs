use crate::{GrowthFunction, Real};
use ndarray::{Array3, Zip};

pub fn map_growth_value(
    potential: Real,
    mu: Real,
    sigma: Real,
    growth_function: GrowthFunction,
) -> Real {
    let safe_sigma = sigma.max(1.0e-4);
    let diff = potential - mu;

    match growth_function {
        GrowthFunction::Exponential => {
            (2.0 * (-(diff * diff) / (2.0 * safe_sigma * safe_sigma)).exp()) - 1.0
        }
        GrowthFunction::Polynomial => {
            let term = 1.0 - (diff * diff) / (9.0 * safe_sigma * safe_sigma);
            2.0 * term.max(0.0).powi(4) - 1.0
        }
        GrowthFunction::Step => {
            if diff.abs() <= safe_sigma {
                1.0
            } else {
                -1.0
            }
        }
    }
}

pub fn apply_growth_mapping(
    input: &Array3<Real>,
    mu: Real,
    sigma: Real,
    growth_function: GrowthFunction,
) -> Array3<Real> {
    let mut output = Array3::zeros(input.dim());
    Zip::from(&mut output)
        .and(input)
        .for_each(|output_value, &input_value| {
            *output_value = map_growth_value(input_value, mu, sigma, growth_function);
        });
    output
}

#[cfg(test)]
mod tests {
    use super::map_growth_value;
    use crate::GrowthFunction;
    use approx::assert_relative_eq;

    #[test]
    fn exponential_growth_peaks_at_mu() {
        let value = map_growth_value(0.35, 0.35, 0.08, GrowthFunction::Exponential);
        assert_relative_eq!(value, 1.0, epsilon = 1.0e-6);
    }

    #[test]
    fn step_growth_is_negative_outside_band() {
        let value = map_growth_value(0.8, 0.2, 0.05, GrowthFunction::Step);
        assert_eq!(value, -1.0);
    }
}
