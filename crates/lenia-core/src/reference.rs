use crate::{
    backend::SimulationBackend, generate_kernel_3d, simulator::step_with_kernel, Kernel3D,
    LeniaParams, World3D,
};

#[derive(Default)]
pub struct ReferenceBackend {
    cached: Option<CachedKernel>,
}

#[derive(Clone)]
struct CachedKernel {
    params: LeniaParams,
    kernel: Kernel3D,
}

impl ReferenceBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_cached_kernel_for(&self, params: &LeniaParams) -> bool {
        self.cached
            .as_ref()
            .is_some_and(|cached| cached.params == *params)
    }

    fn kernel_for(&mut self, params: &LeniaParams) -> Kernel3D {
        if let Some(cached) = &self.cached {
            if cached.params == *params {
                return cached.kernel.clone();
            }
        }

        let kernel = generate_kernel_3d(params);
        self.cached = Some(CachedKernel {
            params: params.clone(),
            kernel: kernel.clone(),
        });
        kernel
    }
}

impl SimulationBackend for ReferenceBackend {
    fn name(&self) -> &'static str {
        "reference"
    }

    fn step(&mut self, world: &World3D, params: &LeniaParams) -> World3D {
        let kernel = self.kernel_for(params);
        step_with_kernel(world, params, kernel.view())
    }
}

#[cfg(test)]
mod tests {
    use super::ReferenceBackend;
    use crate::{LeniaParams, SimulationBackend, World3D};

    #[test]
    fn caches_kernel_for_identical_params() {
        let mut backend = ReferenceBackend::new();
        let params = LeniaParams::default();
        let world = World3D::zeros(8, 8, 8);

        assert!(!backend.has_cached_kernel_for(&params));
        let _ = backend.step(&world, &params);
        assert!(backend.has_cached_kernel_for(&params));
    }
}
