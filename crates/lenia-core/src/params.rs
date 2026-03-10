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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelCore {
    Polynomial,
    Exponential,
    Step,
    Staircase,
}

impl KernelCore {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Polynomial => "POLYNOMIAL",
            Self::Exponential => "EXPONENTIAL",
            Self::Step => "STEP",
            Self::Staircase => "STAIRCASE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelMode {
    GaussianShells,
    CenteredGaussian,
    LeniaBands,
}

impl KernelMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GaussianShells => "GAUSSIAN_SHELLS",
            Self::CenteredGaussian => "CENTERED_GAUSSIAN",
            Self::LeniaBands => "LENIA_BANDS",
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
    pub kernel_mode: KernelMode,
    pub kernel_core: KernelCore,
    pub radius_cells: usize,
    pub mu: Real,
    pub sigma: Real,
    pub time_step: Real,
    pub growth_function: GrowthFunction,
    pub shells: Vec<KernelShell>,
    pub bands: Vec<Real>,
}

impl Default for LeniaParams {
    fn default() -> Self {
        Self {
            kernel_mode: KernelMode::GaussianShells,
            kernel_core: KernelCore::Exponential,
            radius_cells: 6,
            mu: 0.35,
            sigma: 0.08,
            time_step: 0.1,
            growth_function: GrowthFunction::Exponential,
            shells: vec![
                KernelShell::new(0.28, 0.10, 1.0),
                KernelShell::new(0.62, 0.08, 0.55),
            ],
            bands: vec![1.0],
        }
    }
}

impl LeniaParams {
    pub fn centered_gaussian_preset() -> Self {
        Self {
            kernel_mode: KernelMode::CenteredGaussian,
            kernel_core: KernelCore::Exponential,
            radius_cells: 9,
            mu: 0.30,
            sigma: 0.22,
            time_step: 0.07,
            growth_function: GrowthFunction::Exponential,
            shells: vec![
                KernelShell::new(0.0, 0.10, 1.0),
                KernelShell::new(0.0, 0.22, 0.65),
                KernelShell::new(0.0, 0.38, 0.35),
            ],
            bands: vec![1.0],
        }
    }

    pub fn gaussian_rings_preset() -> Self {
        Self::default()
    }

    pub fn lenia_bands_preset() -> Self {
        Self {
            kernel_mode: KernelMode::LeniaBands,
            kernel_core: KernelCore::Polynomial,
            radius_cells: 10,
            mu: 0.15,
            sigma: 0.016,
            time_step: 0.1,
            growth_function: GrowthFunction::Polynomial,
            shells: vec![KernelShell::new(0.5, 0.15, 1.0)],
            bands: vec![1.0, 0.75, 0.5],
        }
    }

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

    pub fn normalized_bands(&self) -> Vec<Real> {
        let mut bands = if self.bands.is_empty() {
            vec![1.0]
        } else {
            self.bands.clone()
        };

        for band in &mut bands {
            *band = band.max(0.0);
        }

        if bands.iter().all(|&band| band == 0.0) {
            bands.fill(1.0);
        }

        bands
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
    use super::{KernelCore, KernelMode, KernelShell, LeniaParams};

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

    #[test]
    fn centered_gaussian_preset_sets_mode() {
        assert_eq!(
            LeniaParams::centered_gaussian_preset().kernel_mode,
            KernelMode::CenteredGaussian
        );
    }

    #[test]
    fn lenia_bands_preset_uses_official_family() {
        let preset = LeniaParams::lenia_bands_preset();
        assert_eq!(preset.kernel_mode, KernelMode::LeniaBands);
        assert_eq!(preset.kernel_core, KernelCore::Polynomial);
    }

    #[test]
    fn normalized_bands_recover_zero_weights() {
        let params = LeniaParams {
            bands: vec![0.0, -1.0, 0.0],
            ..LeniaParams::default()
        };

        let bands = params.normalized_bands();
        assert_eq!(bands, vec![1.0, 1.0, 1.0]);
    }
}
