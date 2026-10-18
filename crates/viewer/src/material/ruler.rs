// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Ruler material.

use super::bevy_types::*;

/// Ruler.
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct Ruler {
    #[uniform(0)]
    zoom_level: f32,
}

impl Default for Ruler {
    fn default() -> Self {
        Self { zoom_level: 1.0 }
    }
}

impl Ruler {
    fn shader_ref() -> ShaderRef {
        super::shader_ref_from_str("ruler.wgsl")
    }
}

impl Material for Ruler {
    fn fragment_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn vertex_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
