// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Grid entity.

use bevy::{
    asset::Assets,
    camera::visibility::Visibility,
    camera::{Camera, Projection},
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::{Vec2, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::MeshMaterial3d,
    transform::components::Transform,
    window::Window,
};

#[derive(Component)]
pub struct ToggleMe;

use crate::{scene::get_current_zoom_level, view_model::ViewModel};
use crate::{to_bevy::ToBevy, *};

pub fn toggle_grid(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Visibility, With<ToggleMe>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        let mut visibility = query.single_mut().expect("Visible");

        *visibility = match *visibility {
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Visible,
        };
    }
}

pub fn spawn_grid_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<material::Grid>>,
    mut state: ResMut<ViewModel>,
) {
    let plane = Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(1000.0, 1000.0)));
    let mesh_handle = meshes.add(plane);

    let theme = &state.config.theme;

    state.scene.grid_entity = Some(
        commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(materials.add(material::Grid {
                    parameters: material::GridUniform {
                        grid_color: theme.darker.to_bevy(),
                        x_axis_color: theme.bright.to_bevy(),
                        y_axis_color: theme.bright.to_bevy(),
                        ..Default::default()
                    },
                })),
                bevy::picking::Pickable::IGNORE,
                Visibility::Visible,
                ToggleMe,
            ))
            .id(),
    );
}

/// Update grid material according to zoom level.
pub fn update_grid(
    mut materials: ResMut<Assets<material::Grid>>,
    state: Res<ViewModel>,
    proj_query: Query<&Projection, With<Camera>>,
    windows: Query<&Window>,
    mat_query: Query<&mut MeshMaterial3d<material::Grid>>,
) {
    let radius = state.scene.radius;
    let Ok(window) = windows.single() else {
        return;
    };

    for projection in proj_query {
        if let Some(grid) = state.scene.grid_entity
            && let Ok(material) = mat_query.get(grid)
            && let Some(material) = materials.get_mut(material)
        {
            material.parameters.radius = radius;
            material.parameters.zoom_level = window.width().max(window.height())
                / 20.0
                / get_current_zoom_level(projection)
                / radius;
        }
    }
}

pub fn update_grid_on_view_angle_change(
    mut materials: ResMut<Assets<material::Grid>>,
    state: Res<ViewModel>,
    cam_query: Query<(&Transform, &Camera)>,
    mat_query: Query<&mut MeshMaterial3d<material::Grid>>,
) {
    for (transform, _) in cam_query {
        if let Some(grid) = state.scene.grid_entity
            && let Ok(material) = mat_query.get(grid)
            && let Some(material) = materials.get_mut(material)
        {
            material.parameters.view_angle = transform.forward().normalize();
        }
    }
}
