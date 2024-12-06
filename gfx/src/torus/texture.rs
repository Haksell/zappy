use super::{server_link::ServerLink, Torus};
use bevy::prelude::*;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use shared::resource::{Resource, Stone};
use shared::utils::lerp;
use shared::{color::RGB, map::Cell, GameState};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::LazyLock;

const BLACKISH: u8 = 17;
pub const TORUS_TEXTURE_SIZE: usize = 1024;
pub const SVG_SIZE: usize = 1024;

static SVGS: LazyLock<HashMap<Resource, Pixmap>> = LazyLock::new(|| {
    use Stone::*;

    fn load_svg(path: &str) -> Pixmap {
        let svg_data = std::fs::read(path).expect(&format!("Failed to read {path:?}"));
        let options = Options::default();
        let tree = Tree::from_data(&svg_data, &options)
            .expect(&format!("Failed to parse {path:?} as a SVG"));

        let mut pixmap = Pixmap::new(SVG_SIZE as u32, SVG_SIZE as u32).unwrap();

        let scale_x = SVG_SIZE as f32 / tree.size().width();
        let scale_y = SVG_SIZE as f32 / tree.size().height();
        let scale = scale_x.min(scale_y);

        let transform = Transform::from_scale(scale, scale);

        resvg::render(&tree, transform, &mut pixmap.as_mut());
        pixmap
    }

    [
        Resource::Stone(Deraumere),
        Resource::Stone(Linemate),
        Resource::Stone(Mendiane),
        Resource::Stone(Phiras),
        Resource::Stone(Sibur),
        Resource::Stone(Thystame),
        Resource::Nourriture,
    ]
    .iter()
    .map(|&r| (r, load_svg(&format!("gfx/assets/{}.svg", r.alias()))))
    .collect()
});

type Interval2D = ((usize, usize), (usize, usize));

fn write_pixel(data: &mut [u8], x: usize, y: usize, (r, g, b): RGB) {
    data[(y * TORUS_TEXTURE_SIZE + x) * 4] = r;
    data[(y * TORUS_TEXTURE_SIZE + x) * 4 + 1] = g;
    data[(y * TORUS_TEXTURE_SIZE + x) * 4 + 2] = b;
}

fn blend_pixmap_with_texture(
    data: &mut [u8],
    pixmap: &Pixmap,
    ((start_x, end_x), (start_y, end_y)): Interval2D,
) {
    let pixmap_data = pixmap.data();

    for y in start_y..end_y {
        for x in start_x..end_x {
            let tex_index = (y * TORUS_TEXTURE_SIZE + x) * 4;

            let pixmap_y = (y - start_y) * pixmap.height() as usize / (end_y - start_y);
            let pixmap_x = (x - start_x) * pixmap.width() as usize / (end_x - start_x);
            let pixmap_index = (pixmap_y * pixmap.width() as usize + pixmap_x) * 4;

            if pixmap_index < pixmap_data.len() && tex_index < data.len() {
                let (r, g, b, a) = (
                    pixmap_data[pixmap_index],
                    pixmap_data[pixmap_index + 1],
                    pixmap_data[pixmap_index + 2],
                    pixmap_data[pixmap_index + 3],
                );

                let alpha = a as f32 / 255.;
                data[tex_index] = lerp(data[tex_index] as f32, r as f32, alpha) as u8;
                data[tex_index + 1] = lerp(data[tex_index + 1] as f32, g as f32, alpha) as u8;
                data[tex_index + 2] = lerp(data[tex_index + 2] as f32, b as f32, alpha) as u8;
                data[tex_index + 3] = 255;
            }
        }
    }
}

fn fill_background(
    data: &mut [u8],
    ((start_x, end_x), (start_y, end_y)): Interval2D,
    bg_color: RGB,
) {
    for y in start_y..end_y {
        for x in start_x..end_x {
            write_pixel(data, x, y, bg_color);
        }
    }
}

fn fill_cell(data: &mut [u8], cell: &Cell, interval: Interval2D) {
    // TODO: for each nourriture and resource, take a random x and y in the interval and draw circle of appropriate color
    if cell.nourriture > 0 {
        blend_pixmap_with_texture(data, &SVGS[&Resource::Nourriture], interval);
    }
    for (i, &cnt) in cell.stones.iter().enumerate() {
        if cnt > 0 {
            let resource = Resource::try_from(i).unwrap();
            blend_pixmap_with_texture(data, &SVGS[&resource], interval);
        }
    }
}

fn fill_texture(data: &mut [u8], game_state: &Option<GameState>) {
    match &game_state {
        None => data.fill(0),
        Some(game_state) => {
            let w = *game_state.map.width();
            let h = *game_state.map.height();

            for map_y in 0..h {
                let start_y = map_y * TORUS_TEXTURE_SIZE / h;
                let end_y = (map_y + 1) * TORUS_TEXTURE_SIZE / h;
                for map_x in 0..w {
                    let start_x = map_x * TORUS_TEXTURE_SIZE / w;
                    let end_x = (map_x + 1) * TORUS_TEXTURE_SIZE / w;
                    let cell_range = ((start_x, end_x), (start_y, end_y));
                    let bgcolor = if map_y & 1 == map_x & 1 {
                        255
                    } else {
                        BLACKISH
                    };
                    fill_background(data, cell_range, (bgcolor, bgcolor, bgcolor));
                    let cell = &game_state.map.field[map_y][map_x];
                    fill_cell(data, cell, cell_range);
                }
            }
        }
    }
}

pub fn update_texture(
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Handle<StandardMaterial>, With<Torus>>,
    mut images: ResMut<Assets<Image>>,
    server_link: ResMut<ServerLink>,
) {
    if server_link.update.load(Ordering::Relaxed) {
        let handle = query.get_single().unwrap();
        let material = materials.get_mut(handle).unwrap();
        let image_handle = material.base_color_texture.as_mut().unwrap();
        let image = images.get_mut(image_handle).unwrap();
        fill_texture(&mut image.data, &server_link.game_state.lock().unwrap());
        server_link.update.store(false, Ordering::Relaxed);
    }
}
