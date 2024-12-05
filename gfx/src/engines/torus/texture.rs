use super::{server_link::ServerLink, Torus, TEXTURE_SIZE};
use crate::engines::ServerData;
use bevy::prelude::*;
use shared::map::Cell;
use std::sync::atomic::Ordering;

// fn fill_texture(data: &mut [u8], game_state: &ServerData) {
//     let w = game_state.map.width();
//     let h = game_state.map.height();

//     for y in 0..TEXTURE_SIZE {
//         let map_y = y * h / TEXTURE_SIZE;
//         for x in 0..TEXTURE_SIZE {
//             let map_x = x * w / TEXTURE_SIZE;

//             let color = if map_y & 1 == map_x & 1 { 255 } else { 17 };
//             data[(y * TEXTURE_SIZE + x) * 4] = color;
//             data[(y * TEXTURE_SIZE + x) * 4 + 1] = color;
//             data[(y * TEXTURE_SIZE + x) * 4 + 2] = color;
//             data[(y * TEXTURE_SIZE + x) * 4 + 3] = 255;
//         }
//     }
// }

fn fill_cell(
    data: &mut [u8],
    cell: &Cell,
    start_x: usize,
    end_x: usize,
    start_y: usize,
    end_y: usize,
) {
    let c = Color()
    todo!()
}

fn fill_texture(data: &mut [u8], game_state: &ServerData) {
    let w = *game_state.map.width();
    let h = *game_state.map.height();

    for map_y in 0..h {
        let start_y = map_y * TEXTURE_SIZE / h;
        let end_y = (map_y + 1) * TEXTURE_SIZE / h;
        for map_x in 0..w {
            let start_x = map_x * TEXTURE_SIZE / w;
            let end_x = (map_x + 1) * TEXTURE_SIZE / w;
            let background_color = if map_y & 1 == map_x & 1 { 255 } else { 17 };
            for y in start_y..end_y {
                for x in start_x..end_x {
                    data[]
                }
            }
            let cell = &game_state.map.field[map_y][map_x];
            fill_cell(data, cell, start_x, end_x, start_y, end_y);
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
