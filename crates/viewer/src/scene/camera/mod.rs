// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod camera_controller;

use bevy::prelude::*;

use crate::ViewModel;

pub fn setup_camera(mut commands: Commands, state: Res<ViewModel>) {
    let radius = state.scene.radius;

    // Place the camera on the +X, +Y, +Z diagonal of the sphere
    let direction = bevy::prelude::Vec3::new(1.0, -1.0, 1.0).normalize() * radius;

    // camera
    commands.spawn((
        Camera3d::default(),
        camera_controller::CameraController::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 2.0 * radius,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(direction)
            .looking_at(bevy::prelude::Vec3::ZERO, bevy::prelude::Vec3::Z),
    ));
}
