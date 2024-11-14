// TODO: better lights
// TODO: share most of the code with 2D bevy Renderer
// TODO: button to swap main axis (probably a bad idea)
// TODO: button to switch from 2D to torus and vice-versa?
// TODO: rotations x/y
// TODO: ESPAAAAAAAAAACE
// TODO: optimize mesh (right now every corner appears 4 times) (mabe unimportant for reasons)
// TODO: do everything with respect to delta time

use bevy::{
    app::App,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    window::WindowResolution,
};
use crossterm::event::KeyEvent;
use rand::{rngs::StdRng, Rng, SeedableRng as _};
use shared::{player::Player, utils::lerp, Map};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

// TODO: read from server
const GRID_U: u8 = 12;
const GRID_V: u8 = 8;

// looks cool with 1 too, make it an argument?
const SUBDIVISIONS: u32 = 10; // TODO: depends on grid width and height, can be different in u and v

const ROTATION_STEPS: u16 = 60; // TODO: depend on delta time instead

#[derive(Resource, Default, Debug)]
struct Keys {
    up: bool,
    right: bool,
    down: bool,
    left: bool,
}

#[derive(Resource, Debug)]
struct TorusTransform {
    minor_angle: u16,
    major_angle: u16,
    minor_radius: f32,
}

impl Default for TorusTransform {
    fn default() -> Self {
        Self {
            minor_angle: 0,
            major_angle: 0,
            minor_radius: 0.4,
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    torus_transform: Res<TorusTransform>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 4.2).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }); // TODO: depends on minor rotation

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3e5,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(2.0, 3.0, 2.0),
        ..Default::default()
    });

    let mut rng = StdRng::seed_from_u64(42);
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
    let v_start =
        v as f32 / GRID_V as f32 + torus_transform.major_angle as f32 / ROTATION_STEPS as f32;
    let v_end = v_start + 1.0 / GRID_V as f32;

    let u_start =
        u as f32 / GRID_U as f32 + torus_transform.minor_angle as f32 / ROTATION_STEPS as f32;
    let u_end = u_start + 1.0 / GRID_U as f32;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for v in 0..=SUBDIVISIONS {
        let v_ratio = lerp(v_start, v_end, v as f32 / SUBDIVISIONS as f32);
        let phi = ((v_ratio * std::f32::consts::TAU) % std::f32::consts::TAU
            + std::f32::consts::TAU)
            % std::f32::consts::TAU;
        let (sin_phi, cos_phi) = phi.sin_cos();

        for u in 0..=SUBDIVISIONS {
            let u_ratio = lerp(u_start, u_end, u as f32 / SUBDIVISIONS as f32);
            let theta = ((u_ratio * std::f32::consts::TAU) % std::f32::consts::TAU
                + std::f32::consts::TAU)
                % std::f32::consts::TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let r = 1.0 + torus_transform.minor_radius * cos_theta;
            let tx = r * cos_phi;
            let ty = r * sin_phi;
            let tz = torus_transform.minor_radius * sin_theta;

            positions.push([tx, ty, tz]);
            normals.push([cos_theta * cos_phi, cos_theta * sin_phi, sin_theta]);
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
    mesh.insert_indices(Indices::U32(indices));
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
) {
    for mouse_event in mouse_wheel_events.read() {
        if let MouseScrollUnit::Pixel = mouse_event.unit {
            println!("ACHTUNG !!!!! {:?}", mouse_event); // TODO: test on different computers and remove
        };
        torus_transform.minor_radius =
            (torus_transform.minor_radius + (mouse_event.y * 0.04)).clamp(0.05, 0.95);
    }
}

fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut keys: ResMut<Keys>,
    mut torus_transform: ResMut<TorusTransform>,
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

    torus_transform.major_angle = (torus_transform.major_angle as i16 + keys.left as i16
        - keys.right as i16
        + ROTATION_STEPS as i16) as u16
        % ROTATION_STEPS;
    torus_transform.minor_angle = (torus_transform.minor_angle as i16 + keys.up as i16
        - keys.down as i16
        + ROTATION_STEPS as i16) as u16
        % ROTATION_STEPS;
}
