// TODO: better lights
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis (probably a bad idea)
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: rotations x/y
// TODO: ESPAAAAAAAAAACE
// TODO: optimize mesh (right now every corner appears 4 times) (mabe unimportant for reasons)

use bevy::{
    app::App,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    window::{RequestRedraw, WindowResolution},
};
use crossterm::event::KeyEvent;
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use shared::{player::Player, Map};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

// TODO: read from server
const GRID_U: u8 = 20;
const GRID_V: u8 = 12;

// TODO: clap arguments
const MAJOR_RADIUS: f32 = 3.0;
const MINOR_RADIUS: f32 = 1.0;

const ROTATION_STEPS: u16 = 60; // TODO: depend on delta time instead

#[derive(Resource, Default, Debug)]
struct Keys {
    up: bool,
    right: bool,
    down: bool,
    left: bool,
}

#[derive(Resource, Debug)]
struct Rotation {
    minor: u16,
    major: u16,
    ratio: i64, // TODO: f32
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            minor: 0,
            major: 0,
            ratio: 0,
        }
    }
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
    fn new(
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
        generate_torus_cell_mesh(&mut mesh, rotation, u, v);
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
                resolution: WindowResolution::new(800., 800.),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_resource::<Rotation>()
        .init_resource::<Keys>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_mouse_wheel,
                handle_keyboard,
                update_cell_mesh
                    .after(handle_mouse_wheel)
                    .after(handle_keyboard),
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
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 13.0).looking_at(Vec3::ZERO, Vec3::Y),
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
            commands.spawn(QuadBundle::new(
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

fn generate_torus_cell_mesh(mesh: &mut Mesh, rotation: &Res<Rotation>, u: u8, v: u8) {
    let v_start = v as f32 / GRID_V as f32 + rotation.major as f32 / ROTATION_STEPS as f32;
    let v_end = v_start + 1.0 / GRID_V as f32;

    let u_start = u as f32 / GRID_U as f32 + rotation.minor as f32 / ROTATION_STEPS as f32;
    let u_end = u_start + 1.0 / GRID_U as f32;

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

            let minor_radius = MINOR_RADIUS + 0.1 * rotation.ratio as f32;

            let x = (MAJOR_RADIUS + minor_radius * cos_theta) * cos_phi;
            let y = (MAJOR_RADIUS + minor_radius * cos_theta) * sin_phi;
            let z = minor_radius * sin_theta;

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

fn update_cell_mesh(
    query: Query<(&Handle<Mesh>, &QuadInfo)>,
    mut meshes: ResMut<Assets<Mesh>>,
    rotation: Res<Rotation>,
) {
    println!("{rotation:?}");
    if !rotation.is_changed() {
        return;
    }
    for (mesh_handle, quad_info) in query.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            generate_torus_cell_mesh(mesh, &rotation, quad_info.u, quad_info.v);
        }
    }
}

fn handle_mouse_wheel(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut rotation: ResMut<Rotation>,
    mut window_event: EventWriter<RequestRedraw>,
) {
    for mouse_event in mouse_wheel_events.read() {
        if let MouseScrollUnit::Pixel = mouse_event.unit {
            println!("ACHTUNG !!!!! {:?}", mouse_event); // TODO: test on different computers and remove
        };
        rotation.minor = (rotation.minor as i16 + mouse_event.y as i16 + ROTATION_STEPS as i16)
            as u16
            % ROTATION_STEPS;
        window_event.send(RequestRedraw);
    }
}

fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut keys: ResMut<Keys>,
    mut rotation: ResMut<Rotation>,
    mut window_event: EventWriter<RequestRedraw>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
        return;
    }

    keys.up = keyboard.pressed(KeyCode::ArrowUp);
    keys.right = keyboard.pressed(KeyCode::ArrowRight);
    keys.down = keyboard.pressed(KeyCode::ArrowDown);
    keys.left = keyboard.pressed(KeyCode::ArrowLeft);
    if keys.left == keys.right && keys.up == keys.down {
        return;
    }

    rotation.major = (rotation.major as i16 + keys.left as i16 - keys.right as i16
        + ROTATION_STEPS as i16) as u16
        % ROTATION_STEPS;
    rotation.ratio += keys.up as i64 - keys.down as i64;
    window_event.send(RequestRedraw);
}
