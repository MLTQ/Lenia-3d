use crate::{Real, World3D};
use ndarray::Array3;

const VALUE_SCALE: Real = 255.0;

pub fn decode_lenia_rle_3d(input: &str) -> Array3<Real> {
    let mut row: Vec<Real> = Vec::new();
    let mut slice: Vec<Vec<Real>> = Vec::new();
    let mut volume: Vec<Vec<Vec<Real>>> = Vec::new();
    let mut count = String::new();
    let mut prefix: Option<char> = None;
    let stream = format!("{}%", input.trim().trim_end_matches('!'));

    for ch in stream.chars() {
        if ch.is_ascii_digit() {
            count.push(ch);
            continue;
        }

        if matches!(ch, 'p'..='y' | '@') {
            prefix = Some(ch);
            continue;
        }

        let repeat = count.parse::<usize>().unwrap_or(1);
        let token = prefix.map_or_else(|| ch.to_string(), |leading| format!("{leading}{ch}"));
        match token.as_str() {
            "$" => {
                append_row(&mut slice, std::mem::take(&mut row), repeat);
            }
            "%" => {
                append_row(&mut slice, std::mem::take(&mut row), repeat);
                append_slice(&mut volume, std::mem::take(&mut slice), repeat);
            }
            _ => {
                let value = decode_lenia_value(&token) / VALUE_SCALE;
                append_value(&mut row, value, repeat);
            }
        }

        count.clear();
        prefix = None;
    }

    cubify_volume(volume)
}

pub fn centered_world_from_rle(world_shape: (usize, usize, usize), cells_rle: &str) -> World3D {
    centered_scaled_world_from_rle(world_shape, cells_rle, 1)
}

pub fn centered_scaled_world_from_rle(
    world_shape: (usize, usize, usize),
    cells_rle: &str,
    scale_factor: usize,
) -> World3D {
    let cells = decode_lenia_rle_3d(cells_rle);
    let cells = scale_cells_nearest(&cells, scale_factor.max(1));
    let (world_depth, world_height, world_width) = world_shape;
    let (seed_depth, seed_height, seed_width) = cells.dim();
    let mut world = World3D::zeros(world_depth, world_height, world_width);
    let z_offset = world_depth as isize / 2 - seed_depth as isize / 2;
    let y_offset = world_height as isize / 2 - seed_height as isize / 2;
    let x_offset = world_width as isize / 2 - seed_width as isize / 2;

    for z in 0..seed_depth {
        for y in 0..seed_height {
            for x in 0..seed_width {
                let value = cells[(z, y, x)];
                if value <= 0.0 {
                    continue;
                }

                let target_z = wrap_index(z_offset + z as isize, world_depth);
                let target_y = wrap_index(y_offset + y as isize, world_height);
                let target_x = wrap_index(x_offset + x as isize, world_width);
                world.set(target_z, target_y, target_x, value);
            }
        }
    }

    world
}

fn scale_cells_nearest(cells: &Array3<Real>, scale_factor: usize) -> Array3<Real> {
    if scale_factor <= 1 {
        return cells.clone();
    }

    let (depth, height, width) = cells.dim();
    let mut scaled = Array3::zeros((
        depth * scale_factor,
        height * scale_factor,
        width * scale_factor,
    ));

    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let value = cells[(z, y, x)];
                for dz in 0..scale_factor {
                    for dy in 0..scale_factor {
                        for dx in 0..scale_factor {
                            scaled[(
                                z * scale_factor + dz,
                                y * scale_factor + dy,
                                x * scale_factor + dx,
                            )] = value;
                        }
                    }
                }
            }
        }
    }

    scaled
}

fn decode_lenia_value(token: &str) -> Real {
    match token {
        "." | "b" => 0.0,
        "o" => VALUE_SCALE,
        _ if token.len() == 1 => {
            let ch = token.chars().next().expect("single-char token");
            (ch as u8 - b'A' + 1) as Real
        }
        _ if token.len() == 2 => {
            let mut chars = token.chars();
            let leading = chars.next().expect("two-char token");
            let trailing = chars.next().expect("two-char token");
            ((leading as u8 - b'p') as usize * 24 + (trailing as u8 - b'A') as usize + 25) as Real
        }
        _ => 0.0,
    }
}

fn append_value(row: &mut Vec<Real>, value: Real, repeat: usize) {
    row.extend(std::iter::repeat_n(value, repeat));
}

fn append_row(slice: &mut Vec<Vec<Real>>, row: Vec<Real>, repeat: usize) {
    slice.push(row);
    for _ in 1..repeat {
        slice.push(Vec::new());
    }
}

fn append_slice(volume: &mut Vec<Vec<Vec<Real>>>, slice: Vec<Vec<Real>>, repeat: usize) {
    volume.push(slice);
    for _ in 1..repeat {
        volume.push(Vec::new());
    }
}

fn cubify_volume(volume: Vec<Vec<Vec<Real>>>) -> Array3<Real> {
    let depth = volume.len().max(1);
    let height = volume.iter().map(Vec::len).max().unwrap_or(1);
    let width = volume
        .iter()
        .flat_map(|slice| slice.iter().map(Vec::len))
        .max()
        .unwrap_or(1);
    let mut cells = Array3::zeros((depth, height, width));

    for (z, slice) in volume.iter().enumerate() {
        for (y, row) in slice.iter().enumerate() {
            for (x, &value) in row.iter().enumerate() {
                cells[(z, y, x)] = value;
            }
        }
    }

    cells
}

fn wrap_index(index: isize, size: usize) -> usize {
    index.rem_euclid(size as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::{centered_scaled_world_from_rle, centered_world_from_rle, decode_lenia_rle_3d};

    #[test]
    fn decodes_simple_three_dimensional_rle() {
        let cells = decode_lenia_rle_3d("A$2B%C$D!");
        assert_eq!(cells.dim(), (2, 2, 2));
        assert!((cells[(0, 0, 0)] - (1.0 / 255.0)).abs() < 1.0e-6);
        assert!((cells[(0, 1, 1)] - (2.0 / 255.0)).abs() < 1.0e-6);
        assert!((cells[(1, 0, 0)] - (3.0 / 255.0)).abs() < 1.0e-6);
        assert!((cells[(1, 1, 0)] - (4.0 / 255.0)).abs() < 1.0e-6);
    }

    #[test]
    fn centers_decoded_seed_into_world() {
        let world = centered_world_from_rle((8, 8, 8), "o!");
        assert_eq!(world.get(4, 4, 4), 1.0);
    }

    #[test]
    fn scaled_centered_seed_expands_footprint() {
        let world = centered_scaled_world_from_rle((8, 8, 8), "o!", 2);
        assert_eq!(world.get(3, 3, 3), 1.0);
        assert_eq!(world.get(4, 4, 4), 1.0);
    }
}
