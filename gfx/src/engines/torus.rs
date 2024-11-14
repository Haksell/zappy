// TODO: better lights
// TODO: handle mouse wheel
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: rotate by dragging or keyboard shortcuts

use bevy::{
    app::App,
    input::mouse::{MouseScrollUnit, MouseWheel},
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

#[derive(Resource, Default, Debug)]
struct Rotation {
    minor: f32,
    major: f32,
}

#[derive(Component)]
struct CameraOrbit {
    angle: f32,
    radius: f32,
}

pub async fn render(
    _event_rx: Receiver<KeyEvent>,
    _rx: Receiver<(Map, HashMap<u16, Player>)>,
    _conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Rotation>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse_wheel)
        .run();
    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let initial_position = Vec3::new(0.0, 5.0, 15.0);
    let radius = Vec3::new(initial_position.x, 0.0, initial_position.z).length();
    let angle = initial_position.z.atan2(initial_position.x);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(initial_position)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        CameraOrbit { angle, radius },
    ));

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

// fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
//     for event in events.read() {
//         match event.0 {
//             Scorer::Ai => score.ai += 1,
//             Scorer::Player => score.player += 1,
//         }
//     }

//     println!("Score: {} - {}", score.player, score.ai);
// }

fn handle_mouse_wheel(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut querya: Query<(&mut Transform, &mut CameraOrbit)>,
    mut rotation: ResMut<Rotation>,
) {
    let rotation_speed = 0.1;

    for event in mouse_wheel_events.read() {
        println!("{:?}", event.unit);
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.1,
        };
        rotation.minor += rotation_speed * delta;

        for (mut transform, mut orbit) in querya.iter_mut() {
            orbit.angle = (orbit.angle + delta * rotation_speed) % std::f32::consts::TAU;
            transform.translation = Vec3::new(
                orbit.radius * orbit.angle.cos(),
                transform.translation.y,
                orbit.radius * orbit.angle.sin(),
            );
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }

    println!("{:?}", rotation);
}
