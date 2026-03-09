use crate::kernel_preview::{draw_radial_kernel_plot, kernel_radial_profile, kernel_slice_image};
use crate::render::{render_world_view, ViewMode};
use crate::viewport3d::VolumeViewport;
use crate::volume_wgpu::VolumeWgpuRenderer;
use eframe::egui::{self, TextureHandle, TextureOptions};
use lenia_core::{
    stamp_gaussian_blob_3d, FftBackend, GrowthFunction, KernelMode, KernelShell, LeniaParams,
    ReferenceBackend, SimulationBackend, World3D,
};
use rand::Rng;

const MIN_WORLD_SIZE: usize = 16;
const MAX_WORLD_SIZE: usize = 160;
const DEFAULT_WORLD_SIZE: usize = 48;
const EXTINCTION_EPSILON: f32 = 1.0e-6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackendKind {
    Reference,
    Fft,
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

pub struct ViewerApp {
    world: World3D,
    params: LeniaParams,
    reference_backend: ReferenceBackend,
    fft_backend: FftBackend,
    backend_kind: BackendKind,
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
    food: FoodSettings,
}

impl ViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            world: World3D::random(DEFAULT_WORLD_SIZE, DEFAULT_WORLD_SIZE, DEFAULT_WORLD_SIZE),
            params: LeniaParams::default(),
            reference_backend: ReferenceBackend::new(),
            fft_backend: FftBackend::new(),
            backend_kind: BackendKind::Fft,
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
        let next = match self.backend_kind {
            BackendKind::Reference => self.reference_backend.step(&self.world, &self.params),
            BackendKind::Fft => self.fft_backend.step(&self.world, &self.params),
        };
        self.world = next;
        self.frame_counter += 1;
        if self.world_is_extinct() {
            self.randomize_world();
            return;
        }
        if self.food.enabled
            && self.food.refresh_period > 0
            && self.frame_counter % self.food.refresh_period == 0
        {
            self.apply_food_sources();
        }
        self.clamp_slice_index();
    }

    fn randomize_world(&mut self) {
        self.world = World3D::random(self.world_size, self.world_size, self.world_size);
        self.frame_counter = 0;
        if !self.food.randomize_each_refresh {
            self.regenerate_food_sources();
        }
        self.apply_food_sources();
        self.clamp_slice_index();
    }

    fn clear_world(&mut self) {
        self.world.fill(0.0);
        self.frame_counter = 0;
        self.apply_food_sources();
    }

    fn resize_world(&mut self) {
        self.world = World3D::random(self.world_size, self.world_size, self.world_size);
        self.slice_index = self.world_size / 2;
        self.frame_counter = 0;
        self.regenerate_food_sources();
        self.apply_food_sources();
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
        if !self.food.enabled {
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
        let image = kernel_slice_image(&self.params);
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
        ui.label(format!("Backend: {}", self.backend_kind.label()));
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
            if ui.button("Randomize").clicked() {
                self.randomize_world();
            }
            if ui.button("Clear").clicked() {
                self.clear_world();
            }
        });
        ui.add(egui::Slider::new(&mut self.steps_per_frame, 1..=8).text("steps / frame"));

        ui.separator();
        ui.collapsing("Backend", |ui| {
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
        });

        ui.separator();
        ui.collapsing("World", |ui| {
            ui.add(
                egui::Slider::new(&mut self.world_size, MIN_WORLD_SIZE..=MAX_WORLD_SIZE)
                    .text("edge")
                    .step_by(8.0),
            );
            self.world_size -= self.world_size % 8;
            self.world_size = self.world_size.max(MIN_WORLD_SIZE);
            if ui.button("Resize and Randomize").clicked() {
                self.resize_world();
            }
        });

        ui.separator();
        ui.collapsing("Food", |ui| {
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
                egui::Slider::new(&mut self.food.refresh_period, 1..=240).text("refresh period"),
            );
            if ui
                .add(egui::Slider::new(&mut self.food.source_count, 1..=64).text("food sources"))
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
        ui.collapsing("Lenia Parameters", |ui| {
            egui::ComboBox::from_label("kernel mode")
                .selected_text(self.params.kernel_mode.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.params.kernel_mode,
                        KernelMode::GaussianShells,
                        KernelMode::GaussianShells.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.params.kernel_mode,
                        KernelMode::CenteredGaussian,
                        KernelMode::CenteredGaussian.as_str(),
                    );
                });

            ui.horizontal(|ui| {
                if ui.button("Shell Preset").clicked() {
                    self.apply_gaussian_rings_preset();
                }
                if ui.button("Gaussian Preset").clicked() {
                    self.apply_centered_gaussian_preset();
                }
            });

            ui.add(egui::Slider::new(&mut self.params.radius_cells, 1..=16).text("radius"));
            ui.add(
                egui::Slider::new(&mut self.params.mu, 0.0..=1.0)
                    .text("mu")
                    .step_by(0.001),
            );
            ui.add(
                egui::Slider::new(&mut self.params.sigma, 0.01..=0.5)
                    .text("sigma")
                    .step_by(0.001),
            );
            ui.add(
                egui::Slider::new(&mut self.params.time_step, 0.001..=0.25)
                    .text("dt")
                    .step_by(0.001),
            );

            egui::ComboBox::from_label("growth")
                .selected_text(self.params.growth_function.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.params.growth_function,
                        GrowthFunction::Exponential,
                        GrowthFunction::Exponential.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.params.growth_function,
                        GrowthFunction::Polynomial,
                        GrowthFunction::Polynomial.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.params.growth_function,
                        GrowthFunction::Step,
                        GrowthFunction::Step.as_str(),
                    );
                });
        });

        ui.separator();
        ui.collapsing("Kernel Components", |ui| {
            ui.label(match self.params.kernel_mode {
                KernelMode::GaussianShells => {
                    "Shell mode uses the center/width/weight of each radial lobe."
                }
                KernelMode::CenteredGaussian => {
                    "Centered Gaussian mode ignores shell centers and stacks center-aligned widths."
                }
            });

            for (index, shell) in self.params.shells.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.label(match self.params.kernel_mode {
                        KernelMode::GaussianShells => format!("Shell {}", index + 1),
                        KernelMode::CenteredGaussian => format!("Gaussian {}", index + 1),
                    });
                    if self.params.kernel_mode == KernelMode::GaussianShells {
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
                    self.params.shells.push(KernelShell::new(0.75, 0.08, 0.4));
                }
                if ui
                    .add_enabled(
                        self.params.shells.len() > 1,
                        egui::Button::new("Remove Last"),
                    )
                    .clicked()
                {
                    self.params.shells.pop();
                }
            });
        });

        ui.separator();
        ui.heading("Kernel Preview");
        ui.label(match self.params.kernel_mode {
            KernelMode::GaussianShells => {
                "Center slice heatmap and full 3D radial average for the shell-based kernel."
            }
            KernelMode::CenteredGaussian => {
                "Center slice heatmap and full 3D radial average for the centered Gaussian stack."
            }
        });

        let radial_profile = kernel_radial_profile(&self.params);
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
    use lenia_core::World3D;

    #[test]
    fn clamp_slice_index_respects_mode_limits() {
        let mut app = ViewerApp {
            world: World3D::zeros(4, 5, 6),
            params: lenia_core::LeniaParams::default(),
            reference_backend: lenia_core::ReferenceBackend::new(),
            fft_backend: lenia_core::FftBackend::new(),
            backend_kind: super::BackendKind::Fft,
            view_mode: ViewMode::SliceYz,
            slice_index: 99,
            running: false,
            steps_per_frame: 1,
            frame_counter: 0,
            world_size: 6,
            texture: None,
            kernel_texture: None,
            volume_viewport: VolumeViewport::default(),
            volume_renderer: None,
            food: super::FoodSettings::default(),
        };

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
        let mut app = ViewerApp {
            world: World3D::zeros(8, 8, 8),
            params: lenia_core::LeniaParams::default(),
            reference_backend: lenia_core::ReferenceBackend::new(),
            fft_backend: lenia_core::FftBackend::new(),
            backend_kind: super::BackendKind::Fft,
            view_mode: ViewMode::Volume3d,
            slice_index: 0,
            running: false,
            steps_per_frame: 1,
            frame_counter: 0,
            world_size: 8,
            texture: None,
            kernel_texture: None,
            volume_viewport: VolumeViewport::default(),
            volume_renderer: None,
            food: super::FoodSettings {
                source_count: 1,
                source_positions: vec![(4, 4, 4)],
                ..super::FoodSettings::default()
            },
        };

        app.apply_food_sources();
        assert!(app.world.mean() > 0.0);
    }

    #[test]
    fn extinct_worlds_auto_reseed_on_step() {
        let mut app = ViewerApp {
            world: World3D::zeros(8, 8, 8),
            params: lenia_core::LeniaParams::default(),
            reference_backend: lenia_core::ReferenceBackend::new(),
            fft_backend: lenia_core::FftBackend::new(),
            backend_kind: super::BackendKind::Reference,
            view_mode: ViewMode::Volume3d,
            slice_index: 0,
            running: false,
            steps_per_frame: 1,
            frame_counter: 7,
            world_size: 8,
            texture: None,
            kernel_texture: None,
            volume_viewport: VolumeViewport::default(),
            volume_renderer: None,
            food: super::FoodSettings {
                enabled: false,
                ..super::FoodSettings::default()
            },
        };

        app.step_once();

        assert!(app.world.view().iter().copied().fold(0.0_f32, f32::max) > 0.0);
        assert_eq!(app.frame_counter, 0);
    }
}
