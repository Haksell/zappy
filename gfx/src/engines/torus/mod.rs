// TODO: better lights
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis (probably a bad idea)
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: ESPAAAAAAAAAACE
// TODO: fix reconnection server

mod events;
mod mesh;
mod server_link;
mod texture;

use crate::Message;

use super::ServerData;
use bevy::{
    app::App,
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    window::WindowResolution,
};
use events::{handle_keyboard, handle_mouse_wheel};
use mesh::{fill_torus_mesh, update_torus_mesh};
use server_link::{network_setup, ServerLink};
use shared::PROJECT_NAME;
use texture::update_texture;
use tokio::sync::mpsc::UnboundedReceiver;

const SUBDIVISIONS: &[u16] = &[8, 13, 21, 34, 55, 89, 144, 233];

const TEXTURE_SIZE: usize = 1280; // TODO: in texture.rs

#[derive(Resource, Debug)]
struct TorusTransform {
    shift_minor: f32,
    shift_major: f32,
    minor_radius: f32,
    subdiv_idx: usize,
    rotate_x: f32,
    rotate_y: f32,
}

impl Default for TorusTransform {
    fn default() -> Self {
        Self {
            shift_minor: 0.,
            shift_major: 0.,
            // TODO: next two values depend on grid size
            minor_radius: 0.42,
            subdiv_idx: 4,
            rotate_x: 0.,
            rotate_y: 0.,
        }
    }
}

#[derive(Component, Debug)]
struct Torus;

pub async fn render(data_rx: UnboundedReceiver<Message>) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: PROJECT_NAME.into(),
                resolution: WindowResolution::new(800., 800.), // TODO: consts + hud
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_resource::<TorusTransform>()
        .insert_resource(ServerLink::new(data_rx))
        .add_systems(Startup, (setup, network_setup))
        .add_systems(
            Update,
            (
                handle_mouse_wheel,
                handle_keyboard,
                update_texture,
                update_torus_mesh
                    .after(handle_mouse_wheel)
                    .after(handle_keyboard),
            ),
        )
        .run();
    Ok(())
}

fn camera_distance(minor_radius: f32) -> f32 {
    2.8 * (1. + minor_radius)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    torus_transform: Res<TorusTransform>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., camera_distance(torus_transform.minor_radius))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3e5,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(2.0, 3.0, 2.0),
        ..Default::default()
    });

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    fill_torus_mesh(&mut mesh, &torus_transform);

    let mut texture = Image::new(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; 4 * TEXTURE_SIZE * TEXTURE_SIZE],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    texture.sampler = ImageSampler::nearest();
    let texture_handle = images.add(texture);

    let material = StandardMaterial {
        base_color_texture: Some(texture_handle),
        metallic: 0.5,
        perceptual_roughness: 0.2,
        ..Default::default()
    };

    let pbr = PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(material),
        ..Default::default()
    };
    commands.spawn((pbr, Torus));
}
