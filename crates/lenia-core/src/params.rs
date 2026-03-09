use crate::Real;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrowthFunction {
    Exponential,
    Polynomial,
    Step,
}

impl GrowthFunction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exponential => "EXPONENTIAL",
            Self::Polynomial => "POLYNOMIAL",
            Self::Step => "STEP",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KernelShell {
    pub center: Real,
    pub width: Real,
    pub weight: Real,
}

impl KernelShell {
    pub fn new(center: Real, width: Real, weight: Real) -> Self {
        Self {
            center,
            width,
            weight,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LeniaParams {
    pub radius_cells: usize,
    pub mu: Real,
    pub sigma: Real,
    pub time_step: Real,
    pub growth_function: GrowthFunction,
    pub shells: Vec<KernelShell>,
}

impl Default for LeniaParams {
    fn default() -> Self {
        Self {
            radius_cells: 6,
            mu: 0.35,
            sigma: 0.08,
            time_step: 0.1,
            growth_function: GrowthFunction::Exponential,
            shells: vec![
                KernelShell::new(0.28, 0.10, 1.0),
                KernelShell::new(0.62, 0.08, 0.55),
            ],
        }
    }
}

impl LeniaParams {
    pub fn kernel_diameter(&self) -> usize {
        self.radius_cells.saturating_mul(2).saturating_add(1)
    }

    pub fn normalized_shells(&self) -> Vec<KernelShell> {
        let mut shells = if self.shells.is_empty() {
            vec![KernelShell::new(0.5, 0.15, 1.0)]
        } else {
            self.shells.clone()
        };

        for shell in &mut shells {
            shell.center = shell.center.clamp(0.0, 1.0);
            shell.width = shell.width.max(1.0e-4);
            shell.weight = shell.weight.max(0.0);
        }

        if shells.iter().all(|shell| shell.weight == 0.0) {
            for shell in &mut shells {
                shell.weight = 1.0;
            }
        }

        shells
    }

    pub fn safe_sigma(&self) -> Real {
        self.sigma.max(1.0e-4)
    }

    pub fn safe_time_step(&self) -> Real {
        self.time_step.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{KernelShell, LeniaParams};

    #[test]
    fn normalized_shells_recover_zero_weights() {
        let params = LeniaParams {
            shells: vec![
                KernelShell::new(0.2, 0.0, 0.0),
                KernelShell::new(2.0, -1.0, 0.0),
            ],
            ..LeniaParams::default()
        };

        let shells = params.normalized_shells();
        assert_eq!(shells.len(), 2);
        assert!(shells.iter().all(|shell| shell.weight > 0.0));
        assert!(shells.iter().all(|shell| shell.width > 0.0));
        assert!(shells
            .iter()
            .all(|shell| (0.0..=1.0).contains(&shell.center)));
    }
}
