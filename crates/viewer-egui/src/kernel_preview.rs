use crate::render::colorize_value;
use eframe::egui::{self, ColorImage, Sense};
use lenia_core::{generate_kernel_3d, LeniaParams};

pub fn kernel_slice_image(params: &LeniaParams) -> ColorImage {
    let kernel = generate_kernel_3d(params);
    let view = kernel.view();
    let (_, height, width) = kernel.shape();
    let center_z = params.radius_cells.min(kernel.shape().0.saturating_sub(1));
    let max_value = view.iter().copied().fold(0.0_f32, f32::max).max(1.0e-12);

    let mut pixels = Vec::with_capacity(width * height);
    for row in 0..height {
        for col in 0..width {
            pixels.push(colorize_value(view[(center_z, row, col)] / max_value));
        }
    }

    ColorImage {
        size: [width, height],
        pixels,
        source_size: egui::vec2(width as f32, height as f32),
    }
}

pub fn kernel_radial_profile(params: &LeniaParams) -> Vec<f32> {
    let kernel = generate_kernel_3d(params);
    let view = kernel.view();
    let center = params.radius_cells as f32;
    let max_radius = params.radius_cells.max(1);
    let mut sums = vec![0.0_f32; max_radius + 1];
    let mut counts = vec![0usize; max_radius + 1];

    for ((z, y, x), value) in view.indexed_iter() {
        let dz = z as f32 - center;
        let dy = y as f32 - center;
        let dx = x as f32 - center;
        let radius = (dx * dx + dy * dy + dz * dz).sqrt().round() as usize;
        if radius <= max_radius {
            sums[radius] += *value;
            counts[radius] += 1;
        }
    }

    sums.into_iter()
        .zip(counts)
        .map(|(sum, count)| if count > 0 { sum / count as f32 } else { 0.0 })
        .collect()
}

pub fn draw_radial_kernel_plot(ui: &mut egui::Ui, profile: &[f32], size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let max_value = profile.iter().copied().fold(0.0_f32, f32::max);

    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::StrokeKind::Outside,
    );

    if profile.len() < 2 || max_value <= 0.0 {
        return;
    }

    let plot_rect = rect.shrink2(egui::vec2(10.0, 10.0));
    painter.line_segment(
        [
            egui::pos2(plot_rect.left(), plot_rect.bottom()),
            egui::pos2(plot_rect.right(), plot_rect.bottom()),
        ],
        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
    );
    painter.line_segment(
        [
            egui::pos2(plot_rect.left(), plot_rect.top()),
            egui::pos2(plot_rect.left(), plot_rect.bottom()),
        ],
        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
    );

    let points: Vec<egui::Pos2> = profile
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = if profile.len() == 1 {
                plot_rect.left()
            } else {
                egui::lerp(
                    plot_rect.left()..=plot_rect.right(),
                    index as f32 / (profile.len() - 1) as f32,
                )
            };
            let y = egui::lerp(
                plot_rect.bottom()..=plot_rect.top(),
                (*value / max_value).clamp(0.0, 1.0),
            );
            egui::pos2(x, y)
        })
        .collect();

    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
    ));
}

#[cfg(test)]
mod tests {
    use super::{kernel_radial_profile, kernel_slice_image};
    use lenia_core::LeniaParams;

    #[test]
    fn kernel_slice_matches_kernel_diameter() {
        let params = LeniaParams::default();
        let image = kernel_slice_image(&params);
        let diameter = params.kernel_diameter();
        assert_eq!(image.size, [diameter, diameter]);
    }

    #[test]
    fn kernel_profile_has_radius_bins() {
        let params = LeniaParams::default();
        let profile = kernel_radial_profile(&params);
        assert_eq!(profile.len(), params.radius_cells + 1);
    }
}
