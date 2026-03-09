mod app;
mod kernel_preview;
mod render;
mod viewport3d;
mod volume_wgpu;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 920.0])
            .with_min_inner_size([960.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lenia 3D Viewer",
        options,
        Box::new(|cc| Ok(Box::new(app::ViewerApp::new(cc)))),
    )
}
