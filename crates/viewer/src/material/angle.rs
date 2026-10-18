// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Angle material.

use super::bevy_types::*;

/// A shader material to draw an angle measure.
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct Angle {
    #[uniform(0)]
    start_angle: f32,

    #[uniform(1)]
    end_angle: f32,

    #[uniform(2)]
    inner_radius: f32,

    #[uniform(3)]
    outer_radius: f32,

    alpha_mode: AlphaMode,
}

impl Default for Angle {
    fn default() -> Self {
        use cgmath::{Deg, Rad};
        Self {
            start_angle: Rad::from(Deg(45.0)).0,
            end_angle: Rad::from(Deg(135.0)).0,
            inner_radius: 0.0,
            outer_radius: 1.0,
            alpha_mode: bevy::prelude::AlphaMode::Blend,
        }
    }
}

impl Material for Angle {
    fn fragment_shader() -> ShaderRef {
        super::shader_ref_from_str("angle.wgsl")
    }

    fn vertex_shader() -> ShaderRef {
        super::shader_ref_from_str("angle.wgsl")
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
