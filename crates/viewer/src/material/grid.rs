// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Grid material.

use bevy::render::render_resource::*;

use super::bevy_types::*;

/// A colored zoom-adaptive grid and fade out radius.
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct Grid {
    /// Grid parameters
    #[uniform(0)]
    pub parameters: GridUniform,
}

/// Parameters for a colored zoom-adaptive grid and fade out radius.
///
/// This struct defines the data that will be passed to your shader.
#[derive(Clone, ShaderType, Debug, Default)]
pub struct GridUniform {
    /// Grid fade out radius.
    pub radius: f32,

    /// Zoom level (defines the tile size of the grid).
    pub zoom_level: f32,

    /// Camera view angle.
    pub view_angle: Vec3,

    /// RGB color of the grid.
    pub grid_color: Vec3,

    /// RGB Color for the x axis.
    pub x_axis_color: Vec3,

    /// RGB Color for the y axis.
    pub y_axis_color: Vec3,
}

impl Grid {
    fn shader_ref() -> ShaderRef {
        super::shader_ref_from_str("grid.wgsl")
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            parameters: GridUniform {
                radius: 1.0,
                zoom_level: 1.0,
                view_angle: Vec3::new(0.0, 0.0, 1.0),
                grid_color: Vec3::new(0.7, 0.7, 0.7),
                x_axis_color: Vec3::new(1.0, 0.0, 0.0),
                y_axis_color: Vec3::new(0.0, 1.0, 0.0),
            },
        }
    }
}

impl Material for Grid {
    fn fragment_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn vertex_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None; // ✅ Disable backface culling
        Ok(())
    }
}
