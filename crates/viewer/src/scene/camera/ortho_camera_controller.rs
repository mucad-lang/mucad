// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Instant;

use crate::{
    asset::Scalar,
    scene::camera::{Vec2, Vec3, ortho_zoom_model::OrthoZoomModel},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Inactive,
    Orbit,
    Translate,
    Zoom,
}

pub struct OrthoCameraController {
    scene_radius: Scalar,
    pub scene_center: Vec3,
    pub pivot: Vec3,

    pub orbit_speed: Scalar,
    pub zoom_speed: Scalar,
    pub translate_speed: Scalar,

    pub zoom_model: OrthoZoomModel, // Implement this similar to your `ZoomRectModel`

    pub mode: CameraMode,

    pub camera_position: Vec3,
    pub camera_up: Vec3,
    pub camera_view_center: Vec3,

    timer: std::time::Instant,
}

impl OrthoCameraController {
    pub fn new(scene_radius: Scalar, window_size: (Scalar, Scalar)) -> Self {
        Self {
            scene_radius: 100.0,
            scene_center: Vec3::new(0.0, 0.0, 0.0),
            pivot: Vec3::new(0.0, 0.0, 0.0),

            orbit_speed: 25.0,
            zoom_speed: 25.0,
            translate_speed: 15.0,

            zoom_model: OrthoZoomModel::new(scene_radius, window_size), // placeholder

            mode: CameraMode::Inactive,

            camera_position: Vec3::new(0.0, 0.0, 0.0),
            camera_up: Vec3::unit_z(),
            camera_view_center: Vec3::new(0.0, 0.0, 0.0),

            timer: Instant::now(),
        }
    }

    pub fn orbit(&mut self, pan: Scalar, tilt: Scalar) {
        use cgmath::InnerSpace;
        use cgmath::Rotation;
        use cgmath::Rotation3;
        let view_vector = (self.camera_view_center - self.camera_position).normalize();

        // Reposition the camera to be "back" from the view center along the view vector
        self.camera_position = self.camera_view_center - view_vector * self.scene_radius * 10.0;

        // Vectors from pivot to current camera position and view center
        let v = self.pivot - self.camera_position;
        let w = self.pivot - self.camera_view_center;

        // Pan rotation: around global Y axis
        let pan_axis = Vec3::unit_y();
        let pan_rot = cgmath::Quaternion::from_axis_angle(pan_axis, cgmath::Rad(-pan));

        // Tilt rotation: around the camera's right axis
        let right = view_vector.cross(self.camera_up).normalize();
        let tilt_rot = cgmath::Quaternion::from_axis_angle(right, cgmath::Rad(tilt));

        // Combined rotation (apply tilt after pan)
        let rotation = pan_rot * tilt_rot;

        // Apply the rotation to position and view center
        self.camera_position = self.pivot - rotation.rotate_vector(v);
        self.camera_view_center = self.pivot - rotation.rotate_vector(w);

        // Recompute up vector
        self.camera_up = rotation.rotate_vector(self.camera_up).normalize();
    }

    pub fn translate(&mut self, d: Vec2) {
        self.zoom_model.translate(d);
    }

    fn right(&self) -> Vec3 {
        self.camera_view_center.cross(self.camera_up)
    }
}
