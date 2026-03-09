use crate::Real;
use ndarray::{Array3, ArrayView3, ArrayViewMut3};
use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct World3D {
    cells: Array3<Real>,
}

impl World3D {
    pub fn zeros(depth: usize, height: usize, width: usize) -> Self {
        assert!(
            depth > 0 && height > 0 && width > 0,
            "world dimensions must be positive"
        );
        Self {
            cells: Array3::zeros((depth, height, width)),
        }
    }

    pub fn random(depth: usize, height: usize, width: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = Array3::zeros((depth, height, width));
        for value in &mut cells {
            *value = rng.gen::<Real>();
        }
        Self { cells }
    }

    pub fn from_array(cells: Array3<Real>) -> Self {
        let (depth, height, width) = cells.dim();
        assert!(
            depth > 0 && height > 0 && width > 0,
            "world dimensions must be positive"
        );
        Self { cells }
    }

    pub fn shape(&self) -> (usize, usize, usize) {
        self.cells.dim()
    }

    pub fn mean(&self) -> Real {
        self.cells.sum() / self.cells.len() as Real
    }

    pub fn fill(&mut self, value: Real) {
        self.cells.fill(value.clamp(0.0, 1.0));
    }

    pub fn get(&self, z: usize, y: usize, x: usize) -> Real {
        self.cells[(z, y, x)]
    }

    pub fn set(&mut self, z: usize, y: usize, x: usize, value: Real) {
        self.cells[(z, y, x)] = value.clamp(0.0, 1.0);
    }

    pub fn view(&self) -> ArrayView3<'_, Real> {
        self.cells.view()
    }

    pub fn view_mut(&mut self) -> ArrayViewMut3<'_, Real> {
        self.cells.view_mut()
    }

    pub fn into_array(self) -> Array3<Real> {
        self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::World3D;

    #[test]
    fn fill_and_set_clamp_values() {
        let mut world = World3D::zeros(2, 2, 2);
        world.fill(2.0);
        assert_eq!(world.get(0, 0, 0), 1.0);

        world.set(1, 1, 1, -3.0);
        assert_eq!(world.get(1, 1, 1), 0.0);
    }
}
