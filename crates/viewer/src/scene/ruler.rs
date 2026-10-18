// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use bevy::{
    asset::Assets,
    ecs::system::{Commands, ResMut},
    math::{Vec2, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::MeshMaterial3d,
};

use crate::material;

#[allow(unused)]
pub fn spawn_ruler_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<material::Ruler>>,
) {
    let plane = Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(10.0, 2.0)));
    let mesh_handle = meshes.add(plane);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(material::Ruler::default())),
        bevy::picking::Pickable::IGNORE,
    ));
}
