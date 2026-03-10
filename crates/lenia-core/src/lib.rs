pub mod backend;
pub mod fft;
pub mod field;
pub mod growth;
pub mod kernel;
pub mod multichannel;
pub mod multichannel_fft;
pub mod params;
pub mod reference;
pub mod rle;
pub mod seed;
pub mod simulator;
pub mod species;

pub use backend::SimulationBackend;
pub use fft::FftBackend;
pub use field::World3D;
pub use growth::{apply_growth_mapping, map_growth_value};
pub use kernel::{generate_kernel_3d, Kernel3D};
pub use multichannel::{
    seed_ndkc_starter_world, step_multichannel_reference, KernelRule, MultiChannelParams,
    MultiChannelWorld,
};
pub use multichannel_fft::MultiChannelFftBackend;
pub use params::{GrowthFunction, KernelCore, KernelMode, KernelShell, LeniaParams};
pub use reference::ReferenceBackend;
pub use rle::{centered_scaled_world_from_rle, centered_world_from_rle, decode_lenia_rle_3d};
pub use seed::stamp_gaussian_blob_3d;
pub use simulator::{convolve_periodic_reference, step_reference};
pub use species::{
    scaled_params_for_preset, scaled_seed_shape_for_preset, seeded_world_for_preset,
    seeded_world_for_preset_scaled, single_species_presets, SingleSpeciesPreset,
};

pub type Real = f32;
