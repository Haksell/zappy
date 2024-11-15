// TODO: better lights
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis (probably a bad idea)
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: torus x and y rotation with WASD
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
use crossterm::event::KeyEvent;
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use shared::{utils::lerp, PROJECT_NAME};
use std::f32::consts::TAU;
use tokio::sync::mpsc::Receiver;

// TODO: read from server
const GRID_U: u8 = 12;
const GRID_V: u8 = 8;

const SUBDIVISIONS: &[u16] = &[1, 2, 3, 5, 8, 13, 21, 34];

const ROTATION_SPEED: f32 = 0.8;
const MOUSE_WHEEL_SPEED: f32 = 3.0;

#[derive(Resource, Default, Debug)]
struct Keys {
    up: bool,
    right: bool,
    down: bool,
    left: bool,
    w: bool,
    a: bool,
    s: bool,
    d: bool,
}

#[derive(Resource, Debug)]
struct TorusTransform {
    minor_shift: f32,
    major_shift: f32,
    minor_radius: f32,
    subdiv_idx: usize,
    rotate_x: f32,
    rotate_y: f32,
}

impl Default for TorusTransform {
    fn default() -> Self {
        Self {
            minor_shift: 0.,
            major_shift: 0.,
            // TODO: next two values depend on grid size
            minor_radius: 0.42,
            subdiv_idx: 4,
            rotate_x: 0.,
            rotate_y: 0.,
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
        torus_transform: &Res<TorusTransform>,
        u: u8,
        v: u8,
    ) -> Self {
        // TODO: refactor
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        fill_torus_cell_mesh(&mut mesh, torus_transform, u, v);
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
    mut _event_rx: Receiver<KeyEvent>,
    mut _rx: Receiver<ServerData>,
    mut _conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
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

fn camera_distance(minor_radius: f32) -> f32 {
    2.8 * (1. + minor_radius)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    let mut rng = StdRng::seed_from_u64(0);
    for v in 0..GRID_V {
        for u in 0..GRID_U {
            commands.spawn(QuadBundle::new(
                &mut rng,
                &mut meshes,
                &mut materials,
                &torus_transform,
                u,
                v,
            ));
        }
    }
}

fn fill_torus_cell_mesh(mesh: &mut Mesh, torus_transform: &Res<TorusTransform>, u: u8, v: u8) {
    let ttsd = SUBDIVISIONS[torus_transform.subdiv_idx];

    let v_start = v as f32 / GRID_V as f32 + torus_transform.major_shift;
    let v_end = v_start + 1. / GRID_V as f32;

    let u_start = u as f32 / GRID_U as f32 + torus_transform.minor_shift;
    let u_end = u_start + 1. / GRID_U as f32;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for v in 0..=ttsd {
        let v_ratio = lerp(v_start, v_end, v as f32 / ttsd as f32);
        let phi = v_ratio * TAU;
        let (sin_phi, cos_phi) = phi.sin_cos();

        for u in 0..=ttsd {
            let u_ratio = lerp(u_start, u_end, u as f32 / ttsd as f32);
            let theta = u_ratio * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let r = 1. + torus_transform.minor_radius * cos_theta;

            let tx = r * cos_phi;
            let ty = r * sin_phi;
            let tz = torus_transform.minor_radius * sin_theta;

            positions.push([tx, ty, tz]);
            normals.push([cos_theta * cos_phi, cos_theta * sin_phi, sin_theta]);
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let mut indices = Vec::new();
    for v in 0..ttsd {
        for u in 0..ttsd {
            let i0 = v * (ttsd + 1) + u;
            let i1 = v * (ttsd + 1) + u + 1;
            let i2 = (v + 1) * (ttsd + 1) + u;
            let i3 = (v + 1) * (ttsd + 1) + u + 1;

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
    query: Query<(&Handle<Mesh>, &QuadInfo)>,
    mut meshes: ResMut<Assets<Mesh>>,
    torus_transform: Res<TorusTransform>,
) {
    if torus_transform.is_changed() {
        for (mesh_handle, quad_info) in &query {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                fill_torus_cell_mesh(mesh, &torus_transform, quad_info.u, quad_info.v);
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut keys: ResMut<Keys>,
    mut transform: ResMut<TorusTransform>,
    mut exit: EventWriter<AppExit>,
    time: Res<Time>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
        return;
    }

    let q = keyboard.just_pressed(KeyCode::KeyQ);
    let e = keyboard.just_pressed(KeyCode::KeyE);
    if q != e {
        transform.subdiv_idx = (transform.subdiv_idx as isize - q as isize + e as isize)
            .clamp(0, (SUBDIVISIONS.len() - 1) as isize) as usize;
    }

    keys.up = keyboard.pressed(KeyCode::ArrowUp);
    keys.right = keyboard.pressed(KeyCode::ArrowRight);
    keys.down = keyboard.pressed(KeyCode::ArrowDown);
    keys.left = keyboard.pressed(KeyCode::ArrowLeft);
    keys.w = keyboard.pressed(KeyCode::KeyW);
    keys.a = keyboard.pressed(KeyCode::KeyA);
    keys.s = keyboard.pressed(KeyCode::KeyS);
    keys.d = keyboard.pressed(KeyCode::KeyD);

    let dt = time.delta_seconds();

    fn update_value(val: &mut f32, key_add: bool, key_sub: bool, dt: f32, modulo: f32) {
        let change = key_add as u32 as f32 - key_sub as u32 as f32;
        if change != 0. {
            *val = (*val - change * dt * ROTATION_SPEED * modulo + modulo) % modulo;
        }
    }

    update_value(&mut transform.major_shift, keys.right, keys.left, dt, 1.0);
    update_value(&mut transform.minor_shift, keys.down, keys.up, dt, 1.0);
    update_value(&mut transform.rotate_x, keys.w, keys.s, dt, TAU);
    update_value(&mut transform.rotate_y, keys.a, keys.d, dt, TAU);
}
