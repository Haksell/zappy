// TODO: better lights
// TODO: handle mouse wheel
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: rotate by dragging or keyboard shortcuts

use bevy::{
    app::App,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use crossterm::event::KeyEvent;
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use shared::{player::Player, Map};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub async fn render(
    _event_rx: Receiver<KeyEvent>,
    _rx: Receiver<(Map, HashMap<u16, Player>)>,
    _conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3e6,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(5.0, 8.0, 5.0),
        ..Default::default()
    });

    // TODO: read from server
    let grid_u = 10;
    let grid_v = 6;

    // TODO: clap arguments
    let ring_radius = 3.0;
    let tube_radius = 1.0;

    let mut rng = StdRng::seed_from_u64(42);

    for v in 0..grid_v {
        let v_start = v as f32 / grid_v as f32;
        let v_end = (v + 1) as f32 / grid_v as f32;

        for u in 0..grid_u {
            let u_start = u as f32 / grid_u as f32;
            let u_end = (u + 1) as f32 / grid_u as f32;

            let cell_mesh =
                generate_torus_cell_mesh(ring_radius, tube_radius, u_start, u_end, v_start, v_end);

            let material = StandardMaterial {
                base_color: Color::srgb(rng.gen(), rng.gen(), rng.gen()),
                metallic: 0.5,
                perceptual_roughness: 0.2,
                ..Default::default()
            };

            commands.spawn(PbrBundle {
                mesh: meshes.add(cell_mesh),
                material: materials.add(material),
                ..Default::default()
            });
        }
    }
}

fn generate_torus_cell_mesh(
    ring_radius: f32,
    tube_radius: f32,
    u_start: f32,
    u_end: f32,
    v_start: f32,
    v_end: f32,
) -> Mesh {
    // looks cool with 1 too, make it an argument?
    const SUBDIVISIONS: u32 = 10; // TODO: depends on grid width and height, can be different in u and v

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for v in 0..=SUBDIVISIONS {
        let v_ratio = v_start + (v_end - v_start) * (v as f32 / SUBDIVISIONS as f32);
        let phi = v_ratio * std::f32::consts::TAU;
        let (sin_phi, cos_phi) = phi.sin_cos();

        for u in 0..=SUBDIVISIONS {
            let u_ratio = u_start + (u_end - u_start) * (u as f32 / SUBDIVISIONS as f32);
            let theta = u_ratio * std::f32::consts::TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            let x = (ring_radius + tube_radius * cos_theta) * cos_phi;
            let y = (ring_radius + tube_radius * cos_theta) * sin_phi;
            let z = tube_radius * sin_theta;

            positions.push([x, y, z]);
            normals.push([cos_theta * cos_phi, cos_theta * sin_phi, sin_theta]);
            uvs.push([u_ratio, v_ratio]);
        }
    }

    let mut indices = Vec::new();
    for v in 0..SUBDIVISIONS {
        for u in 0..SUBDIVISIONS {
            let i0 = v * (SUBDIVISIONS + 1) + u;
            let i1 = v * (SUBDIVISIONS + 1) + u + 1;
            let i2 = (v + 1) * (SUBDIVISIONS + 1) + u;
            let i3 = (v + 1) * (SUBDIVISIONS + 1) + u + 1;

            indices.push(i0 as u32);
            indices.push(i2 as u32);
            indices.push(i1 as u32);

            indices.push(i1 as u32);
            indices.push(i2 as u32);
            indices.push(i3 as u32);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
