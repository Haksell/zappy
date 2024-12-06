use super::{camera_distance, TorusTransform, SUBDIVISIONS};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::CameraRenderGraph,
};
use std::f32::consts::TAU;

const ROTATION_SPEED: f32 = 0.8;
const MOUSE_WHEEL_SPEED: f32 = 3.0;

pub fn handle_mouse_wheel(
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

pub fn handle_keyboard(
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
            *val = (*val - change * dt * ROTATION_SPEED * modulo).rem_euclid(modulo);
        }
    }

    let dt = time.delta_seconds();
    update_value(&mut tt.shift_major, &keys, ArrowRight, ArrowLeft, dt, 1.);
    update_value(&mut tt.shift_minor, &keys, ArrowDown, ArrowUp, dt, 1.);
    update_value(&mut tt.rotate_x, &keys, KeyW, KeyS, dt, TAU);
    update_value(&mut tt.rotate_y, &keys, KeyA, KeyD, dt, TAU);
}
