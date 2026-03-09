use eframe::egui::{Color32, ColorImage};
use lenia_core::World3D;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Volume3d,
    SliceXy,
    SliceXz,
    SliceYz,
    MipXy,
    MipXz,
    MipYz,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Volume3d => "Volume 3D",
            Self::SliceXy => "Slice XY",
            Self::SliceXz => "Slice XZ",
            Self::SliceYz => "Slice YZ",
            Self::MipXy => "MIP XY",
            Self::MipXz => "MIP XZ",
            Self::MipYz => "MIP YZ",
        }
    }

    pub fn slice_axis_label(self) -> &'static str {
        match self {
            Self::Volume3d => "camera",
            Self::SliceXy => "z",
            Self::SliceXz => "y",
            Self::SliceYz => "x",
            Self::MipXy | Self::MipXz | Self::MipYz => "projection",
        }
    }

    pub fn uses_slice_index(self) -> bool {
        matches!(self, Self::SliceXy | Self::SliceXz | Self::SliceYz)
    }

    pub fn requires_texture(self) -> bool {
        !matches!(self, Self::Volume3d)
    }

    pub fn slice_limit(self, world: &World3D) -> usize {
        let (depth, height, width) = world.shape();
        match self {
            Self::Volume3d => 1,
            Self::SliceXy => depth,
            Self::SliceXz => height,
            Self::SliceYz => width,
            Self::MipXy | Self::MipXz | Self::MipYz => 1,
        }
    }
}

pub fn render_world_view(world: &World3D, view_mode: ViewMode, slice_index: usize) -> ColorImage {
    let view = world.view();
    let (depth, height, width) = world.shape();

    let (image_width, image_height) = match view_mode {
        ViewMode::Volume3d => unreachable!("Volume3d is rendered by viewport3d.rs"),
        ViewMode::SliceXy | ViewMode::MipXy => (width, height),
        ViewMode::SliceXz | ViewMode::MipXz => (width, depth),
        ViewMode::SliceYz | ViewMode::MipYz => (height, depth),
    };

    let mut pixels = Vec::with_capacity(image_width * image_height);
    for row in 0..image_height {
        for col in 0..image_width {
            let value = match view_mode {
                ViewMode::Volume3d => unreachable!("Volume3d is rendered by viewport3d.rs"),
                ViewMode::SliceXy => {
                    let z = slice_index.min(depth.saturating_sub(1));
                    view[(z, row, col)]
                }
                ViewMode::SliceXz => {
                    let y = slice_index.min(height.saturating_sub(1));
                    view[(row, y, col)]
                }
                ViewMode::SliceYz => {
                    let x = slice_index.min(width.saturating_sub(1));
                    view[(row, col, x)]
                }
                ViewMode::MipXy => (0..depth)
                    .map(|z| view[(z, row, col)])
                    .fold(0.0_f32, f32::max),
                ViewMode::MipXz => (0..height)
                    .map(|y| view[(row, y, col)])
                    .fold(0.0_f32, f32::max),
                ViewMode::MipYz => (0..width)
                    .map(|x| view[(row, col, x)])
                    .fold(0.0_f32, f32::max),
            };
            pixels.push(colorize_value(value));
        }
    }

    ColorImage {
        size: [image_width, image_height],
        pixels,
        source_size: eframe::egui::vec2(image_width as f32, image_height as f32),
    }
}

pub fn colorize_value(value: f32) -> Color32 {
    let value = value.clamp(0.0, 1.0);
    let low = [8.0, 12.0, 20.0];
    let mid = [24.0, 128.0, 168.0];
    let high = [246.0, 224.0, 94.0];

    let color = if value < 0.55 {
        blend(low, mid, value / 0.55)
    } else {
        blend(mid, high, (value - 0.55) / 0.45)
    };

    Color32::from_rgb(color[0] as u8, color[1] as u8, color[2] as u8)
}

fn blend(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::{render_world_view, ViewMode};
    use lenia_core::World3D;

    #[test]
    fn slice_xy_dimensions_match_world() {
        let world = World3D::zeros(5, 7, 9);
        let image = render_world_view(&world, ViewMode::SliceXy, 0);
        assert_eq!(image.size, [9, 7]);
    }

    #[test]
    fn mip_yz_dimensions_match_world() {
        let world = World3D::zeros(5, 7, 9);
        let image = render_world_view(&world, ViewMode::MipYz, 0);
        assert_eq!(image.size, [7, 5]);
    }

    #[test]
    fn volume_mode_skips_texture_path() {
        assert!(!ViewMode::Volume3d.requires_texture());
    }
}
