use crate::{LeniaParams, World3D};

pub trait SimulationBackend {
    fn name(&self) -> &'static str;
    fn step(&mut self, world: &World3D, params: &LeniaParams) -> World3D;
}
