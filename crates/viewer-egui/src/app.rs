use crate::kernel_preview::{draw_radial_kernel_plot, kernel_radial_profile, kernel_slice_image};
use crate::render::{render_world_view, ViewMode};
use crate::viewport3d::VolumeViewport;
use crate::volume_wgpu::VolumeWgpuRenderer;
use eframe::egui::{self, TextureHandle, TextureOptions};
use lenia_core::{
    scaled_params_for_preset, scaled_seed_shape_for_preset, seed_ndkc_starter_world,
    seeded_world_for_preset_scaled, single_species_presets, stamp_gaussian_blob_3d, FftBackend,
    GrowthFunction, KernelCore, KernelMode, KernelShell, LeniaParams, MultiChannelFftBackend,
    MultiChannelParams, MultiChannelWorld, ReferenceBackend, SimulationBackend,
    SingleSpeciesPreset, World3D,
};
use rand::Rng;

const MIN_WORLD_SIZE: usize = 16;
const MAX_WORLD_SIZE: usize = 256;
const WORLD_SIZE_STEP: usize = 8;
const DEFAULT_WORLD_SIZE: usize = 48;
const EXTINCTION_EPSILON: f32 = 1.0e-6;
const MAX_SPECIES_SCALE: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackendKind {
    Reference,
    Fft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SimulationMode {
    SingleChannel,
    MultiChannel,
}

#[derive(Clone, Debug)]
struct FoodSettings {
    enabled: bool,
    randomize_each_refresh: bool,
    refresh_period: usize,
    source_count: usize,
    blob_size: usize,
    blob_amplitude: f32,
    blob_mu: f32,
    blob_sigma: f32,
    source_positions: Vec<(usize, usize, usize)>,
}

impl Default for FoodSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            randomize_each_refresh: false,
            refresh_period: 10,
            source_count: 10,
            blob_size: 11,
            blob_amplitude: 0.10,
            blob_mu: 0.0,
            blob_sigma: 0.28,
            source_positions: Vec::new(),
        }
    }
}

impl BackendKind {
    fn label(self) -> &'static str {
        match self {
            Self::Reference => "Reference",
            Self::Fft => "FFT",
        }
    }
}

impl SimulationMode {
    fn label(self) -> &'static str {
        match self {
            Self::SingleChannel => "Single Channel",
            Self::MultiChannel => "Multi Channel",
        }
    }
}

pub struct ViewerApp {
    world: World3D,
    params: LeniaParams,
    simulation_mode: SimulationMode,
    multi_world: Option<MultiChannelWorld>,
    multi_params: Option<MultiChannelParams>,
    selected_multi_rule: usize,
    multi_fft_backend: MultiChannelFftBackend,
    reference_backend: ReferenceBackend,
    fft_backend: FftBackend,
    backend_kind: BackendKind,
    auto_reseed_on_extinction: bool,
    view_mode: ViewMode,
    slice_index: usize,
    running: bool,
    steps_per_frame: usize,
    frame_counter: usize,
    world_size: usize,
    texture: Option<TextureHandle>,
    kernel_texture: Option<TextureHandle>,
    volume_viewport: VolumeViewport,
    volume_renderer: Option<VolumeWgpuRenderer>,
    species_presets: Vec<SingleSpeciesPreset>,
    selected_species: usize,
    species_scale: usize,
    food: FoodSettings,
}

impl ViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            world: World3D::random(DEFAULT_WORLD_SIZE, DEFAULT_WORLD_SIZE, DEFAULT_WORLD_SIZE),
            params: LeniaParams::default(),
            simulation_mode: SimulationMode::SingleChannel,
            multi_world: None,
            multi_params: None,
            selected_multi_rule: 0,
            multi_fft_backend: MultiChannelFftBackend::new(),
            reference_backend: ReferenceBackend::new(),
            fft_backend: FftBackend::new(),
            backend_kind: BackendKind::Fft,
            auto_reseed_on_extinction: true,
            view_mode: ViewMode::Volume3d,
            slice_index: DEFAULT_WORLD_SIZE / 2,
            running: true,
            steps_per_frame: 1,
            frame_counter: 0,
            world_size: DEFAULT_WORLD_SIZE,
            texture: None,
            kernel_texture: None,
            volume_viewport: VolumeViewport::default(),
            volume_renderer: cc
                .wgpu_render_state
                .as_ref()
                .map(|render_state| VolumeWgpuRenderer::new(render_state)),
            species_presets: single_species_presets(),
            selected_species: 0,
            species_scale: 1,
            food: FoodSettings::default(),
        }
        .initialized()
    }

    fn initialized(mut self) -> Self {
        self.regenerate_food_sources();
        self.apply_food_sources();
        self
    }

    fn apply_centered_gaussian_preset(&mut self) {
        self.params = LeniaParams::centered_gaussian_preset();
    }

    fn apply_gaussian_rings_preset(&mut self) {
        self.params = LeniaParams::gaussian_rings_preset();
    }

    fn apply_lenia_bands_preset(&mut self) {
        self.params = LeniaParams::lenia_bands_preset();
    }

    fn aligned_world_size(requested_size: usize) -> usize {
        let clamped = requested_size.clamp(MIN_WORLD_SIZE, MAX_WORLD_SIZE);
        let aligned = clamped - (clamped % WORLD_SIZE_STEP);
        aligned.max(MIN_WORLD_SIZE)
    }

    fn resize_single_world_preserving_state(&mut self) {
        self.world = centered_resize_world(
            &self.world,
            (self.world_size, self.world_size, self.world_size),
        );
        self.frame_counter = 0;
        if !self.food.randomize_each_refresh {
            self.regenerate_food_sources();
        }
        self.clamp_slice_index();
    }

    fn resize_multichannel_world_preserving_state(&mut self) {
        let Some(multi_world) = self.multi_world.as_ref() else {
            self.apply_ndkc_starter();
            return;
        };

        let new_shape = (self.world_size, self.world_size, self.world_size);
        self.multi_world = Some(centered_resize_multichannel_world(multi_world, new_shape));
        self.frame_counter = 0;
        self.rebuild_display_world_from_multichannel();
        self.clamp_slice_index();
    }

    fn apply_selected_species(&mut self) {
        if self.species_presets.is_empty() {
            return;
        }

        let preset =
            &self.species_presets[self.selected_species.min(self.species_presets.len() - 1)];
        let scale = self.species_scale.max(1);
        self.simulation_mode = SimulationMode::SingleChannel;
        self.multi_world = None;
        self.multi_params = None;
        self.selected_multi_rule = 0;
        self.params = scaled_params_for_preset(preset, scale);
        self.backend_kind = BackendKind::Fft;
        self.world_size = self.recommended_species_world_size(preset, scale);
        self.world = seeded_world_for_preset_scaled(
            (self.world_size, self.world_size, self.world_size),
            preset,
            scale,
        );
        self.slice_index = self.world_size / 2;
        self.frame_counter = 0;
        self.food.enabled = false;
        self.auto_reseed_on_extinction = false;
        self.running = false;
    }

    fn recommended_species_world_size(&self, preset: &SingleSpeciesPreset, scale: usize) -> usize {
        let scaled_shape = scaled_seed_shape_for_preset(preset, scale);
        let longest_axis = scaled_shape.0.max(scaled_shape.1).max(scaled_shape.2);
        let scaled_radius = scaled_params_for_preset(preset, scale).radius_cells;
        let minimum_fit = longest_axis.saturating_add(scaled_radius.saturating_mul(2));
        Self::aligned_world_size(preset.preferred_world_size.max(minimum_fit))
    }

    fn activate_single_channel_mode(&mut self) {
        self.simulation_mode = SimulationMode::SingleChannel;
        self.multi_world = None;
        self.multi_params = None;
        self.selected_multi_rule = 0;
        self.params = LeniaParams::default();
        self.world = World3D::random(self.world_size, self.world_size, self.world_size);
        self.slice_index = self.world_size / 2;
        self.frame_counter = 0;
        self.auto_reseed_on_extinction = true;
        if !self.food.randomize_each_refresh {
            self.regenerate_food_sources();
        }
        self.apply_food_sources();
        self.clamp_slice_index();
    }

    fn apply_ndkc_starter(&mut self) {
        let params = MultiChannelParams::ndkc_starter_preset();
        let world =
            seed_ndkc_starter_world((self.world_size, self.world_size, self.world_size), &params);
        self.simulation_mode = SimulationMode::MultiChannel;
        self.multi_params = Some(params);
        self.multi_world = Some(world);
        self.selected_multi_rule = 0;
        self.slice_index = self.world_size / 2;
        self.frame_counter = 0;
        self.food.enabled = false;
        self.auto_reseed_on_extinction = true;
        self.rebuild_display_world_from_multichannel();
    }

    fn rebuild_display_world_from_multichannel(&mut self) {
        let Some(multi_world) = self.multi_world.as_ref() else {
            return;
        };

        let (depth, height, width) = multi_world.shape();
        let mut display = World3D::zeros(depth, height, width);
        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    let mut value = 0.0_f32;
                    for channel_index in 0..multi_world.channel_count() {
                        value = value.max(multi_world.channel(channel_index).get(z, y, x));
                    }
                    display.set(z, y, x, value);
                }
            }
        }

        self.world = display;
        self.clamp_slice_index();
    }

    fn kernel_preview_params(&self) -> &LeniaParams {
        if self.simulation_mode == SimulationMode::MultiChannel {
            if let Some(multi_params) = self.multi_params.as_ref() {
                if let Some(rule) = multi_params.rules.get(self.selected_multi_rule) {
                    return &rule.params;
                }
            }
        }

        &self.params
    }

    fn backend_label(&self) -> &'static str {
        match self.simulation_mode {
            SimulationMode::SingleChannel => self.backend_kind.label(),
            SimulationMode::MultiChannel => "FFT (multi-rule)",
        }
    }

    fn draw_lenia_param_editor(ui: &mut egui::Ui, params: &mut LeniaParams) {
        egui::ComboBox::from_label("kernel mode")
            .selected_text(params.kernel_mode.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut params.kernel_mode,
                    KernelMode::GaussianShells,
                    KernelMode::GaussianShells.as_str(),
                );
                ui.selectable_value(
                    &mut params.kernel_mode,
                    KernelMode::CenteredGaussian,
                    KernelMode::CenteredGaussian.as_str(),
                );
                ui.selectable_value(
                    &mut params.kernel_mode,
                    KernelMode::LeniaBands,
                    KernelMode::LeniaBands.as_str(),
                );
            });

        if params.kernel_mode == KernelMode::LeniaBands {
            egui::ComboBox::from_label("kernel core")
                .selected_text(params.kernel_core.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut params.kernel_core,
                        KernelCore::Polynomial,
                        KernelCore::Polynomial.as_str(),
                    );
                    ui.selectable_value(
                        &mut params.kernel_core,
                        KernelCore::Exponential,
                        KernelCore::Exponential.as_str(),
                    );
                    ui.selectable_value(
                        &mut params.kernel_core,
                        KernelCore::Step,
                        KernelCore::Step.as_str(),
                    );
                    ui.selectable_value(
                        &mut params.kernel_core,
                        KernelCore::Staircase,
                        KernelCore::Staircase.as_str(),
                    );
                });
        }

        ui.add(egui::Slider::new(&mut params.radius_cells, 1..=16).text("radius"));
        ui.add(
            egui::Slider::new(&mut params.mu, 0.0..=1.0)
                .text("mu")
                .step_by(0.001),
        );
        ui.add(
            egui::Slider::new(&mut params.sigma, 0.01..=0.5)
                .text("sigma")
                .step_by(0.001),
        );
        ui.add(
            egui::Slider::new(&mut params.time_step, 0.001..=0.25)
                .text("dt")
                .step_by(0.001),
        );

        egui::ComboBox::from_label("growth")
            .selected_text(params.growth_function.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut params.growth_function,
                    GrowthFunction::Exponential,
                    GrowthFunction::Exponential.as_str(),
                );
                ui.selectable_value(
                    &mut params.growth_function,
                    GrowthFunction::Polynomial,
                    GrowthFunction::Polynomial.as_str(),
                );
                ui.selectable_value(
                    &mut params.growth_function,
                    GrowthFunction::Step,
                    GrowthFunction::Step.as_str(),
                );
            });
    }

    fn draw_kernel_component_editor(ui: &mut egui::Ui, params: &mut LeniaParams) {
        ui.label(match params.kernel_mode {
            KernelMode::GaussianShells => {
                "Shell mode uses the center/width/weight of each radial lobe."
            }
            KernelMode::CenteredGaussian => {
                "Centered Gaussian mode ignores shell centers and stacks center-aligned widths."
            }
            KernelMode::LeniaBands => {
                "Lenia band mode matches the official ND kernel family: repeated radial bands weighted by the band list and shaped by the selected kernel core."
            }
        });

        if params.kernel_mode == KernelMode::LeniaBands {
            for (index, band) in params.bands.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.label(format!("Band {}", index + 1));
                    ui.add(
                        egui::Slider::new(band, 0.0..=1.5)
                            .text("weight")
                            .step_by(0.001),
                    );
                });
            }

            ui.horizontal(|ui| {
                if ui.button("Add Band").clicked() {
                    params.bands.push(1.0);
                }
                if ui
                    .add_enabled(params.bands.len() > 1, egui::Button::new("Remove Last"))
                    .clicked()
                {
                    params.bands.pop();
                }
            });
            return;
        }

        for (index, shell) in params.shells.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.label(match params.kernel_mode {
                    KernelMode::GaussianShells => format!("Shell {}", index + 1),
                    KernelMode::CenteredGaussian => format!("Gaussian {}", index + 1),
                    KernelMode::LeniaBands => format!("Band {}", index + 1),
                });
                if params.kernel_mode == KernelMode::GaussianShells {
                    ui.add(
                        egui::Slider::new(&mut shell.center, 0.0..=1.0)
                            .text("center")
                            .step_by(0.001),
                    );
                }
                ui.add(
                    egui::Slider::new(&mut shell.width, 0.01..=0.4)
                        .text("width")
                        .step_by(0.001),
                );
                ui.add(
                    egui::Slider::new(&mut shell.weight, 0.0..=2.0)
                        .text("weight")
                        .step_by(0.001),
                );
            });
        }

        ui.horizontal(|ui| {
            if ui.button("Add Shell").clicked() {
                params.shells.push(KernelShell::new(0.75, 0.08, 0.4));
            }
            if ui
                .add_enabled(params.shells.len() > 1, egui::Button::new("Remove Last"))
                .clicked()
            {
                params.shells.pop();
            }
        });
    }

    fn regenerate_food_sources(&mut self) {
        self.food.source_positions.clear();
        let (depth, height, width) = self.world.shape();
        let mut rng = rand::thread_rng();
        for _ in 0..self.food.source_count {
            let z = rng.gen_range(0..depth);
            let y = rng.gen_range(0..height);
            let x = rng.gen_range(0..width);
            self.food.source_positions.push((z, y, x));
        }
    }

    fn step_once(&mut self) {
        match self.simulation_mode {
            SimulationMode::SingleChannel => {
                let next = match self.backend_kind {
                    BackendKind::Reference => {
                        self.reference_backend.step(&self.world, &self.params)
                    }
                    BackendKind::Fft => self.fft_backend.step(&self.world, &self.params),
                };
                self.world = next;
            }
            SimulationMode::MultiChannel => {
                let Some(multi_params) = self.multi_params.as_ref() else {
                    self.apply_ndkc_starter();
                    return;
                };
                let Some(multi_world) = self.multi_world.as_ref() else {
                    self.apply_ndkc_starter();
                    return;
                };

                self.multi_world = Some(self.multi_fft_backend.step(multi_world, multi_params));
                self.rebuild_display_world_from_multichannel();
            }
        }

        self.frame_counter += 1;
        if self.world_is_extinct() {
            if self.auto_reseed_on_extinction {
                match self.simulation_mode {
                    SimulationMode::SingleChannel => self.randomize_world(),
                    SimulationMode::MultiChannel => self.apply_ndkc_starter(),
                }
            } else {
                self.running = false;
            }
            return;
        }
        if self.simulation_mode == SimulationMode::SingleChannel
            && self.food.enabled
            && self.food.refresh_period > 0
            && self.frame_counter % self.food.refresh_period == 0
        {
            self.apply_food_sources();
        }
        self.clamp_slice_index();
    }

    fn randomize_world(&mut self) {
        match self.simulation_mode {
            SimulationMode::SingleChannel => {
                self.world = World3D::random(self.world_size, self.world_size, self.world_size);
                self.frame_counter = 0;
                self.auto_reseed_on_extinction = true;
                if !self.food.randomize_each_refresh {
                    self.regenerate_food_sources();
                }
                self.apply_food_sources();
                self.clamp_slice_index();
            }
            SimulationMode::MultiChannel => self.apply_ndkc_starter(),
        }
    }

    fn clear_world(&mut self) {
        match self.simulation_mode {
            SimulationMode::SingleChannel => {
                self.world.fill(0.0);
                self.frame_counter = 0;
                self.auto_reseed_on_extinction = true;
                self.apply_food_sources();
            }
            SimulationMode::MultiChannel => {
                if let Some(multi_params) = self.multi_params.as_ref() {
                    self.multi_world = Some(MultiChannelWorld::zeros(
                        multi_params.channel_count,
                        self.world_size,
                        self.world_size,
                        self.world_size,
                    ));
                }
                self.world = World3D::zeros(self.world_size, self.world_size, self.world_size);
                self.frame_counter = 0;
            }
        }
    }

    fn resize_world(&mut self) {
        match self.simulation_mode {
            SimulationMode::SingleChannel => {
                self.world = World3D::random(self.world_size, self.world_size, self.world_size);
                self.slice_index = self.world_size / 2;
                self.frame_counter = 0;
                self.auto_reseed_on_extinction = true;
                self.regenerate_food_sources();
                self.apply_food_sources();
            }
            SimulationMode::MultiChannel => self.apply_ndkc_starter(),
        }
    }

    fn resize_world_preserving_state(&mut self) {
        match self.simulation_mode {
            SimulationMode::SingleChannel => self.resize_single_world_preserving_state(),
            SimulationMode::MultiChannel => self.resize_multichannel_world_preserving_state(),
        }
    }

    fn clamp_slice_index(&mut self) {
        let limit = self.view_mode.slice_limit(&self.world);
        self.slice_index = self.slice_index.min(limit.saturating_sub(1));
    }

    fn world_is_extinct(&self) -> bool {
        self.world.view().iter().copied().fold(0.0_f32, f32::max) <= EXTINCTION_EPSILON
    }

    fn place_food_at(&mut self, z: usize, y: usize, x: usize) {
        stamp_gaussian_blob_3d(
            &mut self.world,
            z,
            y,
            x,
            self.food.blob_size,
            self.food.blob_amplitude,
            self.food.blob_mu,
            self.food.blob_sigma,
        );
    }

    fn apply_food_sources(&mut self) {
        if self.simulation_mode == SimulationMode::MultiChannel || !self.food.enabled {
            return;
        }

        if self.food.randomize_each_refresh {
            let (depth, height, width) = self.world.shape();
            let mut rng = rand::thread_rng();
            for _ in 0..self.food.source_count {
                let z = rng.gen_range(0..depth);
                let y = rng.gen_range(0..height);
                let x = rng.gen_range(0..width);
                self.place_food_at(z, y, x);
            }
            return;
        }

        if self.food.source_positions.len() != self.food.source_count {
            self.regenerate_food_sources();
        }

        let positions = self.food.source_positions.clone();
        for (z, y, x) in positions {
            self.place_food_at(z, y, x);
        }
    }

    fn refresh_texture(&mut self, ctx: &egui::Context) {
        if !self.view_mode.requires_texture() {
            return;
        }

        let image = render_world_view(&self.world, self.view_mode, self.slice_index);
        if let Some(texture) = &mut self.texture {
            texture.set(image, TextureOptions::NEAREST);
        } else {
            self.texture = Some(ctx.load_texture("lenia-3d-view", image, TextureOptions::NEAREST));
        }
    }

    fn refresh_kernel_texture(&mut self, ctx: &egui::Context) {
        let image = kernel_slice_image(self.kernel_preview_params());
        if let Some(texture) = &mut self.kernel_texture {
            texture.set(image, TextureOptions::LINEAR);
        } else {
            self.kernel_texture =
                Some(ctx.load_texture("lenia-3d-kernel", image, TextureOptions::LINEAR));
        }
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        let stats = world_stats(&self.world);

        ui.heading("Lenia 3D");
        ui.label(format!("Simulation: {}", self.simulation_mode.label()));
        ui.label(format!("Backend: {}", self.backend_label()));
        ui.label(format!(
            "Extinction: {}",
            if self.auto_reseed_on_extinction {
                "auto-reseed"
            } else {
                "pause"
            }
        ));
        ui.label(format!(
            "Population mean {:.3} | min {:.3} | max {:.3}",
            stats.mean, stats.min, stats.max
        ));
        ui.label(format!("World size: {}^3", self.world_size));

        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .button(if self.running { "Pause" } else { "Play" })
                .clicked()
            {
                self.running = !self.running;
            }
            if ui.button("Step").clicked() {
                self.step_once();
            }
        });
        ui.horizontal(|ui| {
            let reseed_label = match self.simulation_mode {
                SimulationMode::SingleChannel => "Randomize",
                SimulationMode::MultiChannel => "Reseed",
            };
            if ui.button(reseed_label).clicked() {
                self.randomize_world();
            }
            if ui.button("Clear").clicked() {
                self.clear_world();
            }
        });
        ui.add(egui::Slider::new(&mut self.steps_per_frame, 1..=8).text("steps / frame"));

        ui.separator();
        ui.collapsing("Simulation", |ui| {
            ui.label(match self.simulation_mode {
                SimulationMode::SingleChannel => {
                    "Single-channel Lenia with optional food and FFT stepping."
                }
                SimulationMode::MultiChannel => {
                    "Experimental NDKC-style multi-rule stepping. Rendering uses a max-over-channels composite."
                }
            });
            ui.checkbox(&mut self.auto_reseed_on_extinction, "Auto-reseed on extinction");
            if self.simulation_mode == SimulationMode::SingleChannel {
                if ui.button("Load NDKC Starter").clicked() {
                    self.apply_ndkc_starter();
                }
            } else {
                ui.horizontal(|ui| {
                    if ui.button("Reseed NDKC Starter").clicked() {
                        self.apply_ndkc_starter();
                    }
                    if ui.button("Return to Single Channel").clicked() {
                        self.activate_single_channel_mode();
                    }
                });
            }
        });

        if self.simulation_mode == SimulationMode::MultiChannel {
            ui.separator();
            ui.collapsing("Multi-channel Rules", |ui| {
                let Some(multi_params) = self.multi_params.as_mut() else {
                    ui.label("No multichannel state loaded.");
                    return;
                };
                if multi_params.rules.is_empty() {
                    ui.label("This preset has no interaction rules.");
                    return;
                }

                self.selected_multi_rule =
                    self.selected_multi_rule.min(multi_params.rules.len().saturating_sub(1));

                egui::ComboBox::from_label("active rule")
                    .selected_text(format!(
                        "Rule {}",
                        self.selected_multi_rule.saturating_add(1)
                    ))
                    .show_ui(ui, |ui| {
                        for (index, rule) in multi_params.rules.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.selected_multi_rule,
                                index,
                                format!(
                                    "Rule {}: C{} -> C{}",
                                    index + 1,
                                    rule.source_channel,
                                    rule.target_channel
                                ),
                            );
                        }
                    });

                if let Some(rule) = multi_params.rules.get_mut(self.selected_multi_rule) {
                    ui.add(
                        egui::Slider::new(
                            &mut rule.source_channel,
                            0..=multi_params.channel_count.saturating_sub(1),
                        )
                        .text("source"),
                    );
                    ui.add(
                        egui::Slider::new(
                            &mut rule.target_channel,
                            0..=multi_params.channel_count.saturating_sub(1),
                        )
                        .text("target"),
                    );
                    ui.add(
                        egui::Slider::new(&mut rule.weight, -1.0..=1.5)
                            .text("weight")
                            .step_by(0.01),
                    );
                    ui.label(
                        "Each rule convolves one source channel and adds its weighted growth into one target channel.",
                    );
                }
            });
        }

        ui.separator();
        ui.collapsing("Backend", |ui| {
            if self.simulation_mode == SimulationMode::SingleChannel {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.backend_kind,
                        BackendKind::Reference,
                        BackendKind::Reference.label(),
                    );
                    ui.selectable_value(
                        &mut self.backend_kind,
                        BackendKind::Fft,
                        BackendKind::Fft.label(),
                    );
                });
                ui.label(
                    "Use Reference for oracle-correct small worlds, FFT for interactive stepping.",
                );
            } else {
                ui.label("Multichannel mode uses the cached FFT multi-rule backend.");
            }
        });

        ui.separator();
        ui.collapsing("World", |ui| {
            ui.add(
                egui::Slider::new(&mut self.world_size, MIN_WORLD_SIZE..=MAX_WORLD_SIZE)
                    .text("edge")
                    .step_by(WORLD_SIZE_STEP as f64),
            );
            self.world_size = Self::aligned_world_size(self.world_size);
            let resize_label = match self.simulation_mode {
                SimulationMode::SingleChannel => "Resize and Randomize",
                SimulationMode::MultiChannel => "Resize and Reseed",
            };
            ui.horizontal(|ui| {
                if ui.button(resize_label).clicked() {
                    self.resize_world();
                }
                let preserve_label = match self.simulation_mode {
                    SimulationMode::SingleChannel => "Resize and Preserve State",
                    SimulationMode::MultiChannel => "Resize and Preserve Channels",
                };
                if ui.button(preserve_label).clicked() {
                    self.resize_world_preserving_state();
                }
            });
            ui.label("Preserve resize keeps the centered state and only pads or crops the box.");
        });

        ui.separator();
        ui.collapsing("Species", |ui| {
            if self.species_presets.is_empty() {
                ui.label("No species presets available.");
                return;
            }

            let selected = self.selected_species.min(self.species_presets.len() - 1);
            self.selected_species = selected;
            egui::ComboBox::from_label("preset")
                .selected_text(self.species_presets[selected].name)
                .show_ui(ui, |ui| {
                    for (index, preset) in self.species_presets.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_species, index, preset.name);
                    }
                });

            let preset = &self.species_presets[self.selected_species];
            ui.label(preset.source_note);
            ui.add(egui::Slider::new(&mut self.species_scale, 1..=MAX_SPECIES_SCALE).text("scale"));
            ui.label(
                "Scale loads the official seed at higher voxel resolution and multiplies the kernel radius to match.",
            );
            if ui.button("Load Species").clicked() {
                self.apply_selected_species();
            }
            ui.label(
                "Loading a species disables periodic food so the seed evolves as a closed system.",
            );
        });

        ui.separator();
        ui.collapsing("Food", |ui| {
            if self.simulation_mode == SimulationMode::MultiChannel {
                ui.label("Periodic food is disabled in multichannel mode.");
            }
            ui.add_enabled_ui(
                self.simulation_mode == SimulationMode::SingleChannel,
                |ui| {
                    ui.checkbox(&mut self.food.enabled, "Enable periodic food");
                    if ui
                        .checkbox(
                            &mut self.food.randomize_each_refresh,
                            "Randomize locations each refresh",
                        )
                        .changed()
                        && !self.food.randomize_each_refresh
                    {
                        self.regenerate_food_sources();
                    }
                    ui.add(
                        egui::Slider::new(&mut self.food.refresh_period, 1..=240)
                            .text("refresh period"),
                    );
                    if ui
                        .add(
                            egui::Slider::new(&mut self.food.source_count, 1..=64)
                                .text("food sources"),
                        )
                        .changed()
                        && !self.food.randomize_each_refresh
                    {
                        self.regenerate_food_sources();
                    }
                    ui.add(egui::Slider::new(&mut self.food.blob_size, 3..=31).text("blob size"));
                    ui.add(
                        egui::Slider::new(&mut self.food.blob_amplitude, 0.01..=0.6)
                            .text("blob amplitude")
                            .step_by(0.01),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.food.blob_mu, 0.0..=1.2)
                            .text("blob mu")
                            .step_by(0.01),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.food.blob_sigma, 0.01..=0.8)
                            .text("blob sigma")
                            .step_by(0.01),
                    );

                    ui.horizontal(|ui| {
                        if ui.button("Seed food now").clicked() {
                            self.apply_food_sources();
                        }
                        if ui.button("Regenerate sources").clicked() {
                            self.regenerate_food_sources();
                        }
                    });
                },
            );
        });

        ui.separator();
        ui.collapsing("View", |ui| {
            egui::ComboBox::from_label("mode")
                .selected_text(self.view_mode.label())
                .show_ui(ui, |ui| {
                    for mode in [
                        ViewMode::Volume3d,
                        ViewMode::SliceXy,
                        ViewMode::SliceXz,
                        ViewMode::SliceYz,
                        ViewMode::MipXy,
                        ViewMode::MipXz,
                        ViewMode::MipYz,
                    ] {
                        ui.selectable_value(&mut self.view_mode, mode, mode.label());
                    }
                });

            self.clamp_slice_index();
            if self.view_mode.uses_slice_index() {
                let max_index = self.view_mode.slice_limit(&self.world).saturating_sub(1);
                ui.add(
                    egui::Slider::new(&mut self.slice_index, 0..=max_index)
                        .text(self.view_mode.slice_axis_label()),
                );
            } else {
                ui.label(self.view_mode.label());
            }

            if self.view_mode == ViewMode::Volume3d {
                ui.add(
                    egui::Slider::new(&mut self.volume_viewport.visibility_floor, 0.0..=0.3)
                        .text("visibility floor")
                        .step_by(0.01),
                );
                ui.add(
                    egui::Slider::new(&mut self.volume_viewport.opacity_scale, 0.1..=12.0)
                        .text("opacity scale")
                        .step_by(0.1),
                );
                ui.add(
                    egui::Slider::new(&mut self.volume_viewport.step_count, 24..=384)
                        .text("ray steps"),
                );
                ui.label("The GPU raymarcher integrates density through the full 3D texture.");
                if ui.button("Reset Camera").clicked() {
                    self.volume_viewport.reset_camera();
                }
            }
        });

        ui.separator();
        ui.collapsing("Lenia Parameters", |ui| match self.simulation_mode {
            SimulationMode::SingleChannel => {
                ui.horizontal(|ui| {
                    if ui.button("Shell Preset").clicked() {
                        self.apply_gaussian_rings_preset();
                    }
                    if ui.button("Gaussian Preset").clicked() {
                        self.apply_centered_gaussian_preset();
                    }
                    if ui.button("Official Preset").clicked() {
                        self.apply_lenia_bands_preset();
                    }
                });
                Self::draw_lenia_param_editor(ui, &mut self.params);
            }
            SimulationMode::MultiChannel => {
                let Some(multi_params) = self.multi_params.as_mut() else {
                    ui.label("No multichannel parameters loaded.");
                    return;
                };
                if let Some(rule) = multi_params.rules.get_mut(self.selected_multi_rule) {
                    ui.horizontal(|ui| {
                        if ui.button("Shell Preset").clicked() {
                            rule.params = LeniaParams::gaussian_rings_preset();
                        }
                        if ui.button("Gaussian Preset").clicked() {
                            rule.params = LeniaParams::centered_gaussian_preset();
                        }
                        if ui.button("Official Preset").clicked() {
                            rule.params = LeniaParams::lenia_bands_preset();
                        }
                    });
                    Self::draw_lenia_param_editor(ui, &mut rule.params);
                }
            }
        });

        ui.separator();
        ui.collapsing("Kernel Components", |ui| match self.simulation_mode {
            SimulationMode::SingleChannel => {
                Self::draw_kernel_component_editor(ui, &mut self.params)
            }
            SimulationMode::MultiChannel => {
                let Some(multi_params) = self.multi_params.as_mut() else {
                    ui.label("No multichannel parameters loaded.");
                    return;
                };
                if let Some(rule) = multi_params.rules.get_mut(self.selected_multi_rule) {
                    Self::draw_kernel_component_editor(ui, &mut rule.params);
                }
            }
        });

        ui.separator();
        ui.heading("Kernel Preview");
        ui.label(match self.kernel_preview_params().kernel_mode {
            KernelMode::GaussianShells => {
                "Center slice heatmap and full 3D radial average for the shell-based kernel."
            }
            KernelMode::CenteredGaussian => {
                "Center slice heatmap and full 3D radial average for the centered Gaussian stack."
            }
            KernelMode::LeniaBands => {
                "Center slice heatmap and full 3D radial average for the official ND banded kernel."
            }
        });
        if self.simulation_mode == SimulationMode::MultiChannel {
            ui.label("Previewing the currently selected multichannel rule.");
        }

        let radial_profile = kernel_radial_profile(self.kernel_preview_params());
        let wide_layout = ui.available_width() >= 280.0;
        if wide_layout {
            ui.horizontal(|ui| {
                if let Some(texture) = self.kernel_texture.as_ref() {
                    let side = 128.0_f32.min(ui.available_width() * 0.45).max(110.0);
                    ui.add(egui::Image::new((texture.id(), egui::vec2(side, side))));
                }
                let plot_width = ui.available_width().max(110.0);
                draw_radial_kernel_plot(ui, &radial_profile, egui::vec2(plot_width, 128.0));
            });
        } else {
            if let Some(texture) = self.kernel_texture.as_ref() {
                let side = ui.available_width().min(220.0).max(120.0);
                ui.add(egui::Image::new((texture.id(), egui::vec2(side, side))));
            }
            draw_radial_kernel_plot(
                ui,
                &radial_profile,
                egui::vec2(ui.available_width().max(120.0), 128.0),
            );
        }
    }

    fn draw_viewport(&mut self, ui: &mut egui::Ui) {
        if self.view_mode == ViewMode::Volume3d {
            ui.vertical_centered(|ui| {
                ui.heading("Volume 3D");
                ui.label("Drag to orbit. Scroll to zoom.");
                ui.add_space(8.0);
            });
            self.volume_viewport
                .draw(ui, &self.world, self.volume_renderer.as_ref());
            return;
        }

        let Some(texture) = self.texture.as_ref() else {
            ui.label("No texture available");
            return;
        };

        let texture_id = texture.id();
        let texture_size = texture.size_vec2();
        let available = ui.available_size();
        let scale = (available.x / texture_size.x)
            .min(available.y / texture_size.y)
            .max(0.25);
        let image_size = texture_size * scale;

        ui.vertical_centered(|ui| {
            ui.heading(self.view_mode.label());
            ui.add_space(8.0);
            ui.add(egui::Image::new((texture_id, image_size)));
        });
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            for _ in 0..self.steps_per_frame.max(1) {
                self.step_once();
            }
        }

        self.refresh_texture(ctx);
        self.refresh_kernel_texture(ctx);

        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.draw_controls(ui));
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(6, 8, 14))
                    .inner_margin(16.0),
            )
            .show(ctx, |ui| self.draw_viewport(ui));

        ctx.request_repaint();
    }
}

fn centered_resize_world(world: &World3D, new_shape: (usize, usize, usize)) -> World3D {
    let (new_depth, new_height, new_width) = new_shape;
    let (old_depth, old_height, old_width) = world.shape();
    let mut resized = World3D::zeros(new_depth, new_height, new_width);

    let copy_depth = old_depth.min(new_depth);
    let copy_height = old_height.min(new_height);
    let copy_width = old_width.min(new_width);

    let old_z_start = (old_depth - copy_depth) / 2;
    let old_y_start = (old_height - copy_height) / 2;
    let old_x_start = (old_width - copy_width) / 2;
    let new_z_start = (new_depth - copy_depth) / 2;
    let new_y_start = (new_height - copy_height) / 2;
    let new_x_start = (new_width - copy_width) / 2;

    for z in 0..copy_depth {
        for y in 0..copy_height {
            for x in 0..copy_width {
                resized.set(
                    new_z_start + z,
                    new_y_start + y,
                    new_x_start + x,
                    world.get(old_z_start + z, old_y_start + y, old_x_start + x),
                );
            }
        }
    }

    resized
}

fn centered_resize_multichannel_world(
    world: &MultiChannelWorld,
    new_shape: (usize, usize, usize),
) -> MultiChannelWorld {
    let (depth, height, width) = new_shape;
    let mut resized = MultiChannelWorld::zeros(world.channel_count(), depth, height, width);

    for channel_index in 0..world.channel_count() {
        let resized_channel = centered_resize_world(world.channel(channel_index), new_shape);
        *resized.channel_mut(channel_index) = resized_channel;
    }

    resized
}

#[derive(Clone, Copy)]
struct WorldStats {
    mean: f32,
    min: f32,
    max: f32,
}

fn world_stats(world: &World3D) -> WorldStats {
    let mut min = 1.0_f32;
    let mut max = 0.0_f32;
    let mut sum = 0.0_f32;
    let mut count = 0usize;

    for &value in world.view() {
        min = min.min(value);
        max = max.max(value);
        sum += value;
        count += 1;
    }

    WorldStats {
        mean: if count > 0 { sum / count as f32 } else { 0.0 },
        min,
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::{world_stats, ViewerApp};
    use crate::render::ViewMode;
    use crate::viewport3d::VolumeViewport;
    use lenia_core::{
        scaled_seed_shape_for_preset, seed_ndkc_starter_world, MultiChannelParams, World3D,
    };

    fn test_app(world: World3D, world_size: usize) -> ViewerApp {
        ViewerApp {
            world,
            params: lenia_core::LeniaParams::default(),
            simulation_mode: super::SimulationMode::SingleChannel,
            multi_world: None,
            multi_params: None,
            selected_multi_rule: 0,
            multi_fft_backend: lenia_core::MultiChannelFftBackend::new(),
            reference_backend: lenia_core::ReferenceBackend::new(),
            fft_backend: lenia_core::FftBackend::new(),
            backend_kind: super::BackendKind::Fft,
            auto_reseed_on_extinction: true,
            view_mode: ViewMode::Volume3d,
            slice_index: 0,
            running: false,
            steps_per_frame: 1,
            frame_counter: 0,
            world_size,
            texture: None,
            kernel_texture: None,
            volume_viewport: VolumeViewport::default(),
            volume_renderer: None,
            species_presets: lenia_core::single_species_presets(),
            selected_species: 0,
            species_scale: 1,
            food: super::FoodSettings::default(),
        }
    }

    #[test]
    fn clamp_slice_index_respects_mode_limits() {
        let mut app = test_app(World3D::zeros(4, 5, 6), 6);
        app.view_mode = ViewMode::SliceYz;
        app.slice_index = 99;

        app.clamp_slice_index();
        assert_eq!(app.slice_index, 5);
    }

    #[test]
    fn world_stats_reads_expected_extremes() {
        let mut world = World3D::zeros(2, 2, 2);
        world.set(0, 0, 0, 0.25);
        world.set(1, 1, 1, 0.75);
        let stats = world_stats(&world);

        assert_eq!(stats.min, 0.0);
        assert_eq!(stats.max, 0.75);
        assert!(stats.mean > 0.0);
    }

    #[test]
    fn food_sources_increase_world_density() {
        let mut app = test_app(World3D::zeros(8, 8, 8), 8);
        app.food = super::FoodSettings {
            source_count: 1,
            source_positions: vec![(4, 4, 4)],
            ..super::FoodSettings::default()
        };

        app.apply_food_sources();
        assert!(app.world.mean() > 0.0);
    }

    #[test]
    fn extinct_worlds_auto_reseed_on_step() {
        let mut app = test_app(World3D::zeros(8, 8, 8), 8);
        app.backend_kind = super::BackendKind::Reference;
        app.frame_counter = 7;
        app.food = super::FoodSettings {
            enabled: false,
            ..super::FoodSettings::default()
        };

        app.step_once();

        assert!(app.world.view().iter().copied().fold(0.0_f32, f32::max) > 0.0);
        assert_eq!(app.frame_counter, 0);
    }

    #[test]
    fn applying_species_loads_seed_and_disables_food() {
        let mut app = test_app(World3D::zeros(32, 32, 32), 32);
        app.frame_counter = 3;
        app.species_scale = 2;
        let preset = app.species_presets[app.selected_species].clone();
        let scaled_shape = scaled_seed_shape_for_preset(&preset, 2);
        let longest_axis = scaled_shape.0.max(scaled_shape.1).max(scaled_shape.2);
        let expected_world_size = super::ViewerApp::aligned_world_size(
            preset
                .preferred_world_size
                .max(longest_axis + preset.params.radius_cells * 4),
        );

        app.apply_selected_species();

        assert!(app.world.mean() > 0.0);
        assert!(!app.food.enabled);
        assert_eq!(app.frame_counter, 0);
        assert_eq!(app.world_size, expected_world_size);
        assert_eq!(app.backend_kind, super::BackendKind::Fft);
        assert!(!app.auto_reseed_on_extinction);
        assert!(!app.running);
        assert_eq!(
            app.params.radius_cells,
            app.species_presets[app.selected_species]
                .params
                .radius_cells
                * 2
        );
    }

    #[test]
    fn applying_ndkc_starter_switches_to_multichannel_and_populates_display_world() {
        let mut app = test_app(World3D::zeros(32, 32, 32), 32);

        app.apply_ndkc_starter();

        assert_eq!(app.simulation_mode, super::SimulationMode::MultiChannel);
        assert!(app.multi_world.is_some());
        assert!(app.multi_params.is_some());
        assert!(app.world.mean() > 0.0);
        assert!(!app.food.enabled);
    }

    #[test]
    fn rebuilding_display_world_uses_multichannel_max_composite() {
        let params = MultiChannelParams::ndkc_starter_preset();
        let multi_world = seed_ndkc_starter_world((24, 24, 24), &params);
        let mut app = test_app(World3D::zeros(24, 24, 24), 24);
        app.simulation_mode = super::SimulationMode::MultiChannel;
        app.multi_world = Some(multi_world.clone());
        app.multi_params = Some(params);
        app.backend_kind = super::BackendKind::Reference;
        app.food = super::FoodSettings {
            enabled: false,
            ..super::FoodSettings::default()
        };

        app.rebuild_display_world_from_multichannel();

        let composite_max = app.world.view().iter().copied().fold(0.0_f32, f32::max);
        let channel_max = (0..multi_world.channel_count())
            .map(|index| {
                multi_world
                    .channel(index)
                    .view()
                    .iter()
                    .copied()
                    .fold(0.0_f32, f32::max)
            })
            .fold(0.0_f32, f32::max);

        assert_eq!(composite_max, channel_max);
    }

    #[test]
    fn centered_resize_world_pads_existing_state() {
        let mut world = World3D::zeros(4, 4, 4);
        world.set(1, 1, 1, 0.75);

        let resized = super::centered_resize_world(&world, (8, 8, 8));

        assert_eq!(resized.get(3, 3, 3), 0.75);
        assert_eq!(resized.get(4, 4, 4), 0.0);
    }

    #[test]
    fn centered_resize_world_crops_around_center() {
        let mut world = World3D::zeros(8, 8, 8);
        world.set(0, 0, 0, 1.0);
        world.set(4, 4, 4, 0.5);

        let resized = super::centered_resize_world(&world, (4, 4, 4));

        assert_eq!(resized.get(2, 2, 2), 0.5);
        assert_eq!(resized.view().iter().copied().fold(0.0_f32, f32::max), 0.5);
    }

    #[test]
    fn resize_world_preserving_state_keeps_single_channel_mass() {
        let mut app = test_app(World3D::zeros(16, 16, 16), 16);
        app.world.set(8, 8, 8, 0.8);
        app.food.enabled = false;
        app.world_size = 24;

        app.resize_world_preserving_state();

        assert_eq!(app.world.shape(), (24, 24, 24));
        assert_eq!(
            app.world.view().iter().copied().fold(0.0_f32, f32::max),
            0.8
        );
    }
}
