use super::{server_link::ServerLink, Torus, TEXTURE_SIZE};
use bevy::prelude::*;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use shared::{color::RGB, map::Cell, resource::NOURRITURE_COLOR, GameState};
use std::sync::atomic::Ordering;

// static SVGS

// static SVGS: LazyLock<String> = LazyLock::new(|| {
//     // M3 Ultra takes about 16 million years in --release config
//     another_crate::great_question()
// });

type Interval2D = ((usize, usize), (usize, usize));

fn write_pixel(data: &mut [u8], x: usize, y: usize, (r, g, b): RGB) {
    data[(y * TEXTURE_SIZE + x) * 4] = r;
    data[(y * TEXTURE_SIZE + x) * 4 + 1] = g;
    data[(y * TEXTURE_SIZE + x) * 4 + 2] = b;
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

fn fill_cell(data: &mut [u8], cell: &Cell, ((start_x, end_x), (start_y, end_y)): Interval2D) {
    // TODO: for each nourriture and resource, take a random x and y in the interval and draw circle of appropriate color
    if cell.nourriture > 0 {
        for y in start_y..end_y {
            for x in start_x..end_x {
                if y & 1 == x & 1 {
                    write_pixel(data, x, y, NOURRITURE_COLOR.rgb());
                }
            }
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
                let start_y = map_y * TEXTURE_SIZE / h;
                let end_y = (map_y + 1) * TEXTURE_SIZE / h;
                for map_x in 0..w {
                    let start_x = map_x * TEXTURE_SIZE / w;
                    let end_x = (map_x + 1) * TEXTURE_SIZE / w;
                    let cell_range = ((start_x, end_x), (start_y, end_y));
                    let bgcolor = if map_y & 1 == map_x & 1 { 255 } else { 17 };
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

        // Load and render the SVG
        match std::fs::read("gfx/assets/D.svg") {
            Ok(svg_data) => {
                let options = Options::default();
                if let Ok(tree) = Tree::from_data(&svg_data, &options) {
                    // Create a Pixmap for rendering the SVG
                    let mut pixmap = Pixmap::new(TEXTURE_SIZE as u32, TEXTURE_SIZE as u32).unwrap();

                    let translate_x = (TEXTURE_SIZE - tree.size().width() as usize) / 2;
                    let translate_y = (TEXTURE_SIZE - tree.size().height() as usize) / 2;
                    let transform =
                        Transform::from_translate(translate_x as f32, translate_y as f32);

                    // Render the SVG onto the Pixmap
                    resvg::render(&tree, transform, &mut pixmap.as_mut());

                    // Blend the Pixmap with the existing texture data
                    blend_pixmap_with_texture(&mut image.data, &pixmap);
                }
            }
            Err(err) => println!("{:?}", err),
        }

        server_link.update.store(false, Ordering::Relaxed);
    }
}

fn blend_pixmap_with_texture(texture_data: &mut Vec<u8>, pixmap: &Pixmap) {
    let pixmap_data = pixmap.data();

    // Assume RGBA8 format for both texture and pixmap
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let tex_index = (y * TEXTURE_SIZE + x) * 4;
            let pixmap_index = (y * pixmap.width() as usize + x) * 4;

            if pixmap_index < pixmap_data.len() && tex_index < texture_data.len() {
                let (r, g, b, a) = (
                    pixmap_data[pixmap_index],
                    pixmap_data[pixmap_index + 1],
                    pixmap_data[pixmap_index + 2],
                    pixmap_data[pixmap_index + 3],
                );

                // Blend based on alpha
                let alpha = a as f32 / 255.0;
                texture_data[tex_index] =
                    (texture_data[tex_index] as f32 * (1.0 - alpha) + r as f32 * alpha) as u8;
                texture_data[tex_index + 1] =
                    (texture_data[tex_index + 1] as f32 * (1.0 - alpha) + g as f32 * alpha) as u8;
                texture_data[tex_index + 2] =
                    (texture_data[tex_index + 2] as f32 * (1.0 - alpha) + b as f32 * alpha) as u8;
                texture_data[tex_index + 3] = 255; // Set alpha to fully opaque
            }
        }
    }
}
