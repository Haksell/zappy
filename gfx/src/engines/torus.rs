// TODO: better lights
// TODO: handle mouse wheel
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: rotate by dragging or keyboard shortcuts

use bevy::{
    app::App,
    input::mouse::MouseWheel,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    window::RequestRedraw,
};
use crossterm::event::KeyEvent;
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use shared::{player::Player, Map};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

// TODO: read from server
const GRID_U: u8 = 10;
const GRID_V: u8 = 6;

// TODO: clap arguments
const RING_RADIUS: f32 = 3.0;
const TUBE_RADIUS: f32 = 1.0;

#[derive(Resource, Default, Debug, Clone, Copy)]
struct Rotation {
    minor: f32,
    major: f32,
}

#[derive(Component, Debug)]
struct QuadInfo {
    u: u8,
    v: u8,
}

#[derive(Bundle)]
struct QuadBundle {
    pbr: PbrBundle,
    quad_info: QuadInfo,
}

impl QuadBundle {
    fn spawn(
        rng: &mut StdRng,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        rotation: &Res<Rotation>,
        u: u8,
        v: u8,
    ) -> Self {
        // TODO: refactor
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        generate_torus_cell_mesh(&mut mesh, RING_RADIUS, TUBE_RADIUS, u, v, rotation);
        let material = StandardMaterial {
            base_color: Color::srgb(rng.gen(), rng.gen(), rng.gen()),
            metallic: 0.5,
            perceptual_roughness: 0.2,
            ..Default::default()
        };
        let pbr = PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            ..Default::default()
        };
        let quad_info = QuadInfo { u, v };
        Self { pbr, quad_info }
    }
}

pub async fn render(
    _event_rx: Receiver<KeyEvent>,
    _rx: Receiver<(Map, HashMap<u16, Player>)>,
    _conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "zappy".to_string(), // TODO: constant somewhere
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_resource::<Rotation>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_mouse_wheel,
                update_cell_mesh.after(handle_mouse_wheel),
            ),
        )
        .run();
    Ok(())
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    rotation: Res<Rotation>,
) {
    println!("setup {:?}", rotation);

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

    generate_torus_mesh(commands, meshes, materials, rotation);
}

fn generate_torus_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rotation: Res<Rotation>,
) {
    let mut rng = StdRng::seed_from_u64(42);

    for v in 0..GRID_V {
        for u in 0..GRID_U {
            commands.spawn(QuadBundle::spawn(
                &mut rng,
                &mut meshes,
                &mut materials,
                &rotation,
                u,
                v,
            ));
        }
    }
}

fn generate_torus_cell_mesh(
    mesh: &mut Mesh,
    ring_radius: f32,
    tube_radius: f32,
    u: u8,
    v: u8,
    rotation: &Res<Rotation>,
) {
    let v_start = v as f32 / GRID_V as f32 + rotation.minor;
    let v_end = (v + 1) as f32 / GRID_V as f32 + rotation.minor;

    let u_start = u as f32 / GRID_U as f32;
    let u_end = (u + 1) as f32 / GRID_U as f32;

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

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
}

fn update_cell_mesh(mut query: Query<(&mut Handle<Mesh>, &QuadInfo)>, rotation: Res<Rotation>) {
    if !rotation.is_changed() {
        return;
    }
    if let Ok((mesh, quad_info)) = query.get_single_mut() {
        println!("{:?}", mesh);
        println!("{:?}", quad_info);
        println!("{:?}", rotation);
        // generate_torus_mesh()
    }
}

fn handle_mouse_wheel(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut rotation: ResMut<Rotation>,
    mut window_event: EventWriter<RequestRedraw>,
) {
    const ROTATION_SPEED: f32 = 0.1;

    for mouse_event in mouse_wheel_events.read() {
        rotation.minor += ROTATION_SPEED * mouse_event.y.signum();
        window_event.send(RequestRedraw);
    }
}
