// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A freecam-style camera controller plugin.
//! To use in your own application:
//! - Copy the code for the [`CameraControllerPlugin`] and add the plugin to your App.
//! - Attach the [`CameraController`] component to an entity with a [`Camera3d`].
//!
//! Unlike other examples, which demonstrate an application, this demonstrates a plugin library.

use crate::ViewModel;
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

/// A freecam-style camera controller plugin.
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (run_camera_controller, zoom_to_fit));
    }
}

/// Camera controller [`Component`].
#[derive(Component)]
pub struct CameraController {
    /// Enables this [`CameraController`] when `true`.
    pub enabled: bool,
    /// Indicates if this controller has been initialized by the [`CameraControllerPlugin`].
    pub initialized: bool,

    /// [`KeyCode`] for forward translation.
    pub key_forward: KeyCode,
    /// [`KeyCode`] for backward translation.
    pub key_back: KeyCode,
    /// [`KeyCode`] for left translation.
    pub key_left: KeyCode,
    /// [`KeyCode`] for right translation.
    pub key_right: KeyCode,
    /// [`KeyCode`] for up translation.
    pub key_up: KeyCode,
    /// [`KeyCode`] for down translation.
    pub key_down: KeyCode,
    /// [`KeyCode`] to use [`run_speed`](CameraController::run_speed) instead of
    /// [`walk_speed`](CameraController::walk_speed) for translation.
    pub key_run: KeyCode,
    /// [`MouseButton`] for grabbing the mouse focus.
    pub mouse_key_cursor_grab: MouseButton,
    /// [`KeyCode`] for grabbing the keyboard focus.
    pub keyboard_key_toggle_cursor_grab: KeyCode,
    /// Multiplier for unmodified translation speed.
    pub walk_speed: f32,
    /// Multiplier for running translation speed.
    pub run_speed: f32,
    /// Multiplier for how the mouse scroll wheel modifies [`walk_speed`](CameraController::walk_speed)
    /// and [`run_speed`](CameraController::run_speed).
    pub scroll_factor: f32,
    /// Friction factor used to exponentially decay [`velocity`](CameraController::velocity) over time.
    pub friction: f32,
    /// This [`CameraController`]'s pitch rotation.
    pub pitch: f32,
    /// This [`CameraController`]'s yaw rotation.
    pub yaw: f32,
    /// This [`CameraController`]'s target point,
    pub target: Vec3,
    /// This [`CameraController`]'s translation velocity.
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            key_run: KeyCode::ShiftLeft,
            mouse_key_cursor_grab: MouseButton::Left,
            keyboard_key_toggle_cursor_grab: KeyCode::KeyM,
            walk_speed: 5.0,
            run_speed: 15.0,
            scroll_factor: 0.1,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            target: Vec3::ZERO,
            velocity: Vec3::ZERO,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_camera_controller(
    time: Res<Time>,
    mut windows: Query<&mut Window>,
    mut cursor_options: Query<&mut CursorOptions>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut toggle_cursor_grab: Local<bool>,
    mut mouse_cursor_grab: Local<bool>,
    state: Res<ViewModel>,
    mut query: Query<(&mut Projection, &mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_secs();

    let Ok((mut projection, mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    if !controller.initialized {
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YZX);
        controller.yaw = yaw;
        controller.pitch = pitch;
        controller.initialized = true;
    }
    if !controller.enabled {
        return;
    }

    let mut scroll = 0.0;

    let amount = match accumulated_mouse_scroll.unit {
        MouseScrollUnit::Line => accumulated_mouse_scroll.delta.y,
        MouseScrollUnit::Pixel => accumulated_mouse_scroll.delta.y / 16.0,
    };
    scroll += amount;
    controller.walk_speed += scroll * controller.scroll_factor * controller.walk_speed;
    controller.run_speed = controller.walk_speed * 3.0;

    // Handle key input
    let mut axis_input = Vec3::ZERO;
    if key_input.pressed(controller.key_right) {
        axis_input.x += 1.0;
    }
    if key_input.pressed(controller.key_left) {
        axis_input.x -= 1.0;
    }
    if key_input.pressed(controller.key_forward) {
        axis_input.z += 1.0;
    }
    if key_input.pressed(controller.key_back) {
        axis_input.z -= 1.0;
    }
    if key_input.pressed(controller.key_up) {
        scroll += 1.0;
    }
    if key_input.pressed(controller.key_down) {
        scroll -= 1.0;
    }

    let current_scale = match projection.as_mut() {
        Projection::Orthographic(ortho) => {
            // Change the projection parameters
            use bevy::camera::CameraProjection;
            ortho.scale *= 1.0 + scroll / 50.0;
            ortho.far = state.scene.radius * 6.0;
            ortho.scaling_mode = bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 2.0 * state.scene.radius,
            };
            match windows.single() {
                Ok(window) => {
                    ortho.update(window.width(), window.height());
                }
                Err(e) => log::error!("{e}"),
            }
            state.scene.radius * Vec2::new(ortho.scale, ortho.scale)
        }
        _ => {
            unimplemented!("Only orthographic camera is supported")
        }
    };

    let mut cursor_grab_change = false;
    if key_input.just_pressed(controller.keyboard_key_toggle_cursor_grab) {
        *toggle_cursor_grab = !*toggle_cursor_grab;
        cursor_grab_change = true;
    }
    if mouse_button_input.just_pressed(controller.mouse_key_cursor_grab) {
        *mouse_cursor_grab = true;
        cursor_grab_change = true;
    }
    if mouse_button_input.just_released(controller.mouse_key_cursor_grab) {
        *mouse_cursor_grab = false;
        cursor_grab_change = true;
    }
    let cursor_grab = *mouse_cursor_grab || *toggle_cursor_grab;

    // Apply movement update
    if axis_input != Vec3::ZERO {
        let max_speed = if key_input.pressed(controller.key_run) {
            controller.run_speed
        } else {
            controller.walk_speed
        };
        controller.velocity = axis_input.normalize() * max_speed;
    } else {
        let friction = controller.friction.clamp(0.0, 1.0);
        controller.velocity *= 1.0 - friction;
        if controller.velocity.length_squared() < 1e-6 {
            controller.velocity = Vec3::ZERO;
        }
    }
    let forward = *transform.forward();
    let right = *transform.right();
    controller.target = controller.target
        + controller.velocity.x * dt * right
        + controller.velocity.z * dt * Vec3::Z
        + controller.velocity.y * dt * forward;

    // Handle cursor grab
    if cursor_grab_change {
        if cursor_grab {
            for window in &mut windows {
                if !window.focused {
                    continue;
                }
                for mut cursor_option in &mut cursor_options {
                    cursor_option.grab_mode = CursorGrabMode::Locked;
                    cursor_option.visible = false;
                }
            }
        } else {
            for mut cursor_option in &mut cursor_options {
                cursor_option.grab_mode = CursorGrabMode::None;
                cursor_option.visible = true;
            }
        }
    }

    let delta = accumulated_mouse_motion.delta;
    let orbit_distance = state.scene.radius * 3.0;

    // Orbit.
    if mouse_button_input.pressed(MouseButton::Left) {
        let orbit_speed = 0.005;
        let yaw_rot = Quat::from_rotation_z(delta.x * orbit_speed);
        let pitch_rot = Quat::from_rotation_x(-delta.y * orbit_speed);
        transform.rotation = yaw_rot * transform.rotation * pitch_rot;
    }

    // Strafe/translate.
    if mouse_button_input.pressed(MouseButton::Right) {
        let forward = *transform.up() * delta.y * current_scale.y;
        let right = *transform.right() * delta.x * -current_scale.x;
        controller.target = controller.target + dt * right + dt * Vec3::Z + dt * forward;
    }
    transform.translation = controller.target - transform.forward() * orbit_distance;
}

fn zoom_to_fit(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut query: Query<&mut Projection, With<Camera>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Ok(mut projection) = query.single_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    crate::scene::zoom_to_fit(projection.as_mut(), window);
}
