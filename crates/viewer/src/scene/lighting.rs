// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    },
    light::{DirectionalLight, light_consts},
    math::Vec3,
    transform::components::Transform,
};

/// Setup lights
pub fn spawn_lights(mut commands: Commands, mut state: ResMut<crate::view_model::ViewModel>) {
    // Despawn all light entities to remove them from the scene
    for entity in &state.scene.light_entities {
        commands.entity(*entity).despawn();
    }

    let mut entities: Vec<Entity> = Vec::new();
    let radius = state.scene.radius;

    entities.push(
        commands
            .spawn((
                DirectionalLight {
                    illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                    ..Default::default()
                },
                Transform::from_xyz(2.0 * radius, -radius, 3.0 * radius)
                    .looking_at(Vec3::ZERO, Vec3::Z),
            ))
            .id(),
    );
    entities.push(
        commands
            .spawn((
                DirectionalLight {
                    illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                    ..Default::default()
                },
                Transform::from_xyz(-2.0 * radius, radius, -3.0 * radius)
                    .looking_at(Vec3::ZERO, Vec3::Z),
            ))
            .id(),
    );

    state.scene.light_entities = entities;
}
