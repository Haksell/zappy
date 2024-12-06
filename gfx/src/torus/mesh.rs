use super::{Torus, TorusTransform, SUBDIVISIONS};
use bevy::{prelude::*, render::mesh::Indices};
use std::f32::consts::TAU;

pub fn fill_torus_mesh(mesh: &mut Mesh, torus_transform: &Res<TorusTransform>) {
    let subdiv = SUBDIVISIONS[torus_transform.subdiv_idx];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    for y in 0..=subdiv {
        let v_ratio = y as f32 / subdiv as f32;
        let phi = (v_ratio + torus_transform.shift_major) * TAU;
        let (sin_phi, cos_phi) = phi.sin_cos();

        for x in 0..=subdiv {
            let u_ratio = x as f32 / subdiv as f32;
            let theta = (u_ratio + torus_transform.shift_minor) * TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let r = 1. + torus_transform.minor_radius * cos_theta;

            let tx = r * cos_phi;
            let ty = r * sin_phi;
            let tz = torus_transform.minor_radius * sin_theta;

            positions.push([tx, ty, tz]);
            normals.push([cos_theta * cos_phi, cos_theta * sin_phi, sin_theta]);
            uvs.push([v_ratio, 1. - u_ratio]); // (v, 1-u) mapping
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

pub fn update_torus_mesh(
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
