// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad viewer scene elements and routines.

use crate::*;
use bevy::prelude::*;

mod angle;
mod camera;
mod grid;
mod lighting;
mod overlay;
mod ruler;

/// Get current zoom level.
pub fn get_current_zoom_level(projection: &Projection) -> f32 {
    match projection {
        Projection::Orthographic(orthographic_projection) => orthographic_projection.scale,
        _ => todo!(),
    }
}

/// Get current resolution in mm.
pub fn get_current_resolution(projection: &Projection, window: &Window) -> f32 {
    let area_size = match projection {
        Projection::Orthographic(orthographic_projection) => orthographic_projection
            .area
            .width()
            .max(orthographic_projection.area.height()),
        _ => todo!(),
    };

    area_size / window.width().max(window.height()) * 10.0
}

/// Set zoom level to 100%.
pub fn zoom_to_fit(projection: &mut Projection, window: &Window) {
    match projection {
        Projection::Orthographic(ortho) => {
            // Change the projection parameters
            use bevy::camera::CameraProjection;
            ortho.scale = 1.0;
            ortho.update(window.width(), window.height());
        }
        _ => {
            // Not an orthographic camera
        }
    }
}

/// A system that draws hit indicators for every pointer.
pub fn draw_mesh_intersections(
    pointers: Query<&bevy::picking::pointer::PointerInteraction>,
    mut gizmos: Gizmos,
    view_model: Res<ViewModel>,
    projections: Query<&Projection>,
) {
    for (_entity, hit) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
    {
        if let Some(position) = hit.position {
            let proj = projections.single().expect("Some projections");

            let zoom = get_current_zoom_level(proj);
            let color: Color = view_model.config.theme.guide.to_bevy();
            gizmos.sphere(position, zoom * 0.1, color);
        }
    }
}

/// Scene radius.
pub struct Scene {
    /// Radius of scene's bounding sphere.
    pub radius: f32,
    /// The scene grid (will be `Some` after the grid has been spawned).
    pub grid_entity: Option<Entity>,
    /// Light entities.
    pub light_entities: Vec<Entity>,
    /// Model entities.
    pub model_entities: Vec<Entity>,
}

impl Scene {
    /// Minimum radius in mm for a scene.
    pub const MINIMUM_RADIUS: f32 = 10.0;
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            radius: 100.0,
            grid_entity: Default::default(),
            light_entities: Default::default(),
            model_entities: Default::default(),
        }
    }
}

/// The bevy scene plugin contains all systems that are to be rendered except the µcad models.
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(camera::camera_controller::CameraControllerPlugin)
            .add_systems(Update, lighting::spawn_lights)
            .add_systems(Startup, grid::spawn_grid_plane)
            //.add_systems(Startup, angle::spawn_angle_plane)
            //.add_systems(Startup, ruler::spawn_ruler_plane)
            .add_systems(Startup, camera::setup_camera)
            .add_systems(Startup, overlay::setup_overlay)
            .add_systems(Update, draw_mesh_intersections)
            .add_systems(Update, overlay::update_overlay)
            .add_systems(Update, grid::update_grid)
            .add_systems(Update, grid::toggle_grid)
            .add_systems(Update, grid::update_grid_on_view_angle_change);
    }
}
