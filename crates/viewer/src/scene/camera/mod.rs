// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod camera_controller;

use bevy::prelude::*;

use crate::ViewModel;

/// The view of the camera at the start, e.g. from the command line.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraView {
    /// The direction from the model to the camera, e.g. `(1, -1, 1)` from the front right above.
    pub direction: Vec3,
    /// The zoom, `1.0` shows the whole scene, `2.0` the model twice as big.
    pub zoom: f32,
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            direction: Vec3::new(1.0, -1.0, 1.0),
            zoom: 1.0,
        }
    }
}

pub fn setup_camera(mut commands: Commands, state: Res<ViewModel>) {
    let radius = state.scene.radius;
    let view = &state.config.camera_view;

    // Place the camera in the direction of the view, the controller keeps its distance.
    let direction = view
        .direction
        .normalize_or(Vec3::new(1.0, -1.0, 1.0).normalize())
        * radius;

    // camera
    commands.spawn((
        Camera3d::default(),
        camera_controller::CameraController::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 2.0 * radius,
            },
            // A smaller scale shows the model bigger.
            scale: 1.0 / view.zoom,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(direction)
            .looking_at(bevy::prelude::Vec3::ZERO, bevy::prelude::Vec3::Z),
    ));
}
