// TODO: better lights
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis (probably a bad idea)
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: ESPAAAAAAAAAACE
// TODO: optimize mesh (right now every corner appears 4 times) (mabe unimportant for reasons)

use super::ServerData;
use bevy::{
    app::App,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    window::WindowResolution,
};
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use shared::{utils::lerp, PROJECT_NAME};
use std::{
    f32::consts::TAU,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::sync::mpsc::Receiver;

// TODO: read from server
const WIDTH: u8 = 1;
const HEIGHT: u8 = 1;

const SUBDIVISIONS: &[u16] = &[3, 5, 8, 13, 21, 34, 55, 89, 144];

const ROTATION_SPEED: f32 = 0.8;
const MOUSE_WHEEL_SPEED: f32 = 3.0;

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

// TODO: don't clone and lock all this
#[derive(Resource)]
struct ServerLink {
    data_rx: Arc<Mutex<Receiver<ServerData>>>,
    game_state: Arc<Mutex<ServerData>>,
}

impl ServerLink {
    fn new(data_rx: Receiver<ServerData>) -> Self {
        Self {
            data_rx: Arc::new(Mutex::new(data_rx)),
            game_state: Default::default(),
        }
    }
}

#[derive(Component, Debug)]
struct Torus;

pub async fn render(data_rx: Receiver<ServerData>) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: PROJECT_NAME.into(),
                resolution: WindowResolution::new(800., 800.),
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
                update_cell_mesh
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
    torus_transform: Res<TorusTransform>,
    asset_server: Res<AssetServer>,
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

    let texture_handle1: Handle<Image> =
        asset_server.load("/mnt/nfs/homes/axbrisse/Downloads/pacman.png");

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    fill_torus_mesh(&mut mesh, &torus_transform);
    let material = StandardMaterial {
        base_color_texture: Some(texture_handle1.clone()),
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

fn network_setup(server_link: ResMut<ServerLink>) {
    let data_rx = Arc::clone(&server_link.data_rx);
    let game_state = Arc::clone(&server_link.game_state);

    thread::spawn(move || {
        let mut data_rx = data_rx.lock().unwrap();

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                loop {
                    tokio::select! {
                        Some(new_data) = data_rx.recv() => {
                            *game_state.lock().unwrap() = new_data;
                        }
                        // Helps not crashing when closing bevy. TODO: find a better way?
                        _ = tokio::time::sleep(Duration::from_millis(50)) => {} // TODO: check best sleep
                    }
                }
            });
    });
}

fn fill_torus_mesh(mesh: &mut Mesh, torus_transform: &Res<TorusTransform>) {
    let subdiv = SUBDIVISIONS[torus_transform.subdiv_idx];

    let v_start = torus_transform.shift_major;
    let v_end = v_start + 1. / HEIGHT as f32;

    let u_start = torus_transform.shift_minor;
    let u_end = u_start + 1. / WIDTH as f32;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for y in 0..=subdiv {
        let v_ratio = lerp(v_start, v_end, y as f32 / subdiv as f32);
        let phi = v_ratio * TAU;
        let (sin_phi, cos_phi) = phi.sin_cos();

        for x in 0..=subdiv {
            let u_ratio = lerp(u_start, u_end, x as f32 / subdiv as f32);
            let theta = u_ratio * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let r = 1. + torus_transform.minor_radius * cos_theta;

            let tx = r * cos_phi;
            let ty = r * sin_phi;
            let tz = torus_transform.minor_radius * sin_theta;

            positions.push([tx, ty, tz]);
            normals.push([cos_theta * cos_phi, cos_theta * sin_phi, sin_theta]);
            uvs.push([u_ratio, v_ratio]);
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let mut indices = Vec::new();
    for y in 0..subdiv {
        for x in 0..subdiv {
            let i0 = y * (subdiv + 1) + x;
            let i1 = y * (subdiv + 1) + x + 1;
            let i2 = (y + 1) * (subdiv + 1) + x;
            let i3 = (y + 1) * (subdiv + 1) + x + 1;

            indices.push(i0 as u32);
            indices.push(i2 as u32);
            indices.push(i1 as u32);

            indices.push(i1 as u32);
            indices.push(i2 as u32);
            indices.push(i3 as u32);
        }
    }
    mesh.insert_indices(Indices::U32(indices));
    mesh.rotate_by(Quat::from_axis_angle(Vec3::X, torus_transform.rotate_x));
    mesh.rotate_by(Quat::from_axis_angle(Vec3::Y, torus_transform.rotate_y));
}

fn update_cell_mesh(
    query: Query<(&Handle<Mesh>, &Torus)>,
    mut meshes: ResMut<Assets<Mesh>>,
    torus_transform: Res<TorusTransform>,
) {
    if torus_transform.is_changed() {
        if let Ok((mesh_handle, _)) = query.get_single() {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                fill_torus_mesh(mesh, &torus_transform);
            }
        }
    }
}

fn handle_mouse_wheel(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut torus_transform: ResMut<TorusTransform>,
    mut camera_query: Query<&mut Transform, With<CameraRenderGraph>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for mouse_event in mouse_wheel_events.read() {
        if let MouseScrollUnit::Pixel = mouse_event.unit {
            println!("ACHTUNG !!!!! {:?}", mouse_event); // TODO: test on different computers and remove
        };
        torus_transform.minor_radius = (torus_transform.minor_radius
            + (mouse_event.y * dt * MOUSE_WHEEL_SPEED))
            .clamp(0.05, 0.95);

        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation.z = camera_distance(torus_transform.minor_radius);
        }
    }
}

fn handle_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut tt: ResMut<TorusTransform>,
    mut exit: EventWriter<AppExit>,
    time: Res<Time>,
) {
    use KeyCode::*;

    if keys.pressed(Escape) {
        exit.send(AppExit::Success);
        return;
    }

    let q = keys.just_pressed(KeyQ);
    let e = keys.just_pressed(KeyE);
    if q != e {
        tt.subdiv_idx = (tt.subdiv_idx as isize - q as isize + e as isize)
            .clamp(0, (SUBDIVISIONS.len() - 1) as isize) as usize;
    }

    fn update_value(
        val: &mut f32,
        keys: &Res<ButtonInput<KeyCode>>,
        key_add: KeyCode,
        key_sub: KeyCode,
        dt: f32,
        modulo: f32,
    ) {
        let change = keys.pressed(key_add) as u32 as f32 - keys.pressed(key_sub) as u32 as f32;
        if change != 0. {
            *val = (*val - change * dt * ROTATION_SPEED * modulo + modulo) % modulo;
        }
    }

    let dt = time.delta_seconds();
    update_value(&mut tt.shift_major, &keys, ArrowRight, ArrowLeft, dt, 1.);
    update_value(&mut tt.shift_minor, &keys, ArrowDown, ArrowUp, dt, 1.);
    update_value(&mut tt.rotate_x, &keys, KeyW, KeyS, dt, TAU);
    update_value(&mut tt.rotate_y, &keys, KeyA, KeyD, dt, TAU);
}
