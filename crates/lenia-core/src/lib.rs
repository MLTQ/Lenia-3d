pub mod backend;
pub mod fft;
pub mod field;
pub mod growth;
pub mod kernel;
pub mod params;
pub mod reference;
pub mod seed;
pub mod simulator;

pub use backend::SimulationBackend;
pub use fft::FftBackend;
pub use field::World3D;
pub use growth::{apply_growth_mapping, map_growth_value};
pub use kernel::{generate_kernel_3d, Kernel3D};
pub use params::{GrowthFunction, KernelShell, LeniaParams};
pub use reference::ReferenceBackend;
pub use seed::stamp_gaussian_blob_3d;
pub use simulator::{convolve_periodic_reference, step_reference};

pub type Real = f32;
