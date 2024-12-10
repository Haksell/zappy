use super::svg::{PLAYER_SVG, SVGS};
use super::TorusTransform;
use super::{server_link::ServerLink, Torus};
use bevy::prelude::*;
use resvg::tiny_skia::Pixmap;
use shared::cell::CellPos;
use shared::math::lerp;
use shared::resource::{Resource, RESOURCE_PROPORTION};
use shared::{cell::Cell, color::RGB, GFXData};
use std::sync::atomic::Ordering;

type Interval2D = ((usize, usize), (usize, usize));

pub const TORUS_TEXTURE_SIZE: usize = 2048;
pub const TORUS_INTERVAL: Interval2D = ((0, TORUS_TEXTURE_SIZE), (0, TORUS_TEXTURE_SIZE));

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

fn calc_interval(((start_x, end_x), (start_y, end_y)): Interval2D, pos: &CellPos) -> Interval2D {
    let (start_x, end_x) = (
        lerp(start_x as f32, end_x as f32, pos.x - RESOURCE_PROPORTION) as usize,
        lerp(start_x as f32, end_x as f32, pos.x + RESOURCE_PROPORTION) as usize,
    );
    let (start_y, end_y) = (
        lerp(start_y as f32, end_y as f32, pos.y - RESOURCE_PROPORTION) as usize,
        lerp(start_y as f32, end_y as f32, pos.y + RESOURCE_PROPORTION) as usize,
    );
    ((start_x, end_x), (start_y, end_y))
}

fn fill_cell(data: &mut [u8], cell: &Cell, interval: Interval2D) {
    // TODO: for each nourriture and resource, take a random x and y in the interval and draw circle of appropriate color
    // TODO: accept several of same type
    // TODO: in GFXData, mix stone count and nourriture count
    for pos in &cell.nourriture {
        let nourriture_interval = calc_interval(interval, pos);
        blend_pixmap_with_texture(data, &SVGS[&Resource::Nourriture], nourriture_interval);
    }
    for (i, positions) in cell.stones.iter().enumerate() {
        for pos in positions {
            let resource = Resource::try_from(i).unwrap();
            let resource_interval = calc_interval(interval, pos);
            // blend_pixmap_with_texture(data, &SVGS[&resource], resource_interval);
            blend_pixmap_with_texture(data, &PLAYER_SVG, resource_interval);
        }
    }
}

pub fn fill_disconnected(data: &mut [u8]) {
    const DISCONNECTED_COLOR: RGB = (220, 20, 60);
    fill_background(data, TORUS_INTERVAL, DISCONNECTED_COLOR);
}

fn fill_texture(data: &mut [u8], game_state: &Option<GFXData>, blackish: RGB) {
    match &game_state {
        None => fill_disconnected(data),
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
                        (255, 255, 255)
                    } else {
                        blackish
                    };
                    fill_background(data, cell_range, bgcolor);
                    let cell = &game_state.map.field[map_y][map_x];
                    fill_cell(data, cell, cell_range);
                }
            }
        }
    }
}

pub fn update_texture(
    torus_transform: Res<TorusTransform>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Handle<StandardMaterial>, With<Torus>>,
    mut images: ResMut<Assets<Image>>,
    server_link: ResMut<ServerLink>,
) {
    // TODO: remove torus_transform.is_changed()
    // used now for blackish rgb
    if server_link.update.load(Ordering::Relaxed) || torus_transform.is_changed() {
        let handle = query.get_single().unwrap();
        let material = materials.get_mut(handle).unwrap();
        let image_handle = material.base_color_texture.as_mut().unwrap();
        let image = images.get_mut(image_handle).unwrap();
        let game_state = server_link.game_state.lock().unwrap();
        fill_texture(&mut image.data, &game_state, torus_transform.blackish);
        server_link.update.store(false, Ordering::Relaxed);
    }
}
