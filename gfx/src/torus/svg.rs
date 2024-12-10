use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};
use shared::resource::{Resource, Stone};
use std::{collections::HashMap, sync::LazyLock};

const SVG_SIZE: usize = 1024;

fn load_svg(path: &str) -> Pixmap {
    let svg_data = std::fs::read(path).expect(&format!("Failed to read {path:?}"));
    let options = Options::default();
    let tree =
        Tree::from_data(&svg_data, &options).expect(&format!("Failed to parse {path:?} as a SVG"));

    let mut pixmap = Pixmap::new(SVG_SIZE as u32, SVG_SIZE as u32).unwrap();

    let scale_x = SVG_SIZE as f32 / tree.size().width();
    let scale_y = SVG_SIZE as f32 / tree.size().height();
    let scale = scale_x.min(scale_y);

    let transform = Transform::from_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap
}

pub static SVGS: LazyLock<HashMap<Resource, Pixmap>> = LazyLock::new(|| {
    use Stone::*;

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

pub static PLAYER_SVG: LazyLock<Pixmap> =
    LazyLock::new(|| load_svg(&format!("gfx/assets/pacman.svg")));
pub static GHOST_SVG: LazyLock<Pixmap> =
    LazyLock::new(|| load_svg(&format!("gfx/assets/ghost.svg")));
