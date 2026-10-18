// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Conversions from mucad core types to bevy types.

pub mod to_mesh;

pub use to_mesh::ToBevyMesh;

use bevy::{asset::RenderAssetUsages, math::Vec3, platform::collections::HashMap};
use bevy::{
    mesh::{Indices, Mesh},
    transform::components::Transform,
};

pub use mucad_driver::prelude as mu;

/// A trait to convert a µcad type into a bevy type.
pub trait ToBevy<T> {
    /// Convert into bevy type.
    fn to_bevy(self) -> T;
}

/// Convert a µcad color into a bevy color.
impl ToBevy<bevy::prelude::Color> for mu::core::Color {
    fn to_bevy(self) -> bevy::prelude::Color {
        bevy::prelude::Color::srgba(self.r, self.g, self.b, self.a)
    }
}

/// Convert the color of a model attribute (e.g. `#[color = "red"]`) into a bevy color.
impl ToBevy<bevy::prelude::Color> for mu::types::Color {
    fn to_bevy(self) -> bevy::prelude::Color {
        bevy::prelude::Color::srgba(self.r.into(), self.g.into(), self.b.into(), self.a.into())
    }
}

impl ToBevy<bevy::prelude::Vec3> for mu::core::Color {
    fn to_bevy(self) -> bevy::prelude::Vec3 {
        bevy::prelude::Vec3::new(self.r, self.g, self.b)
    }
}

impl ToBevy<bevy::prelude::Mat4> for mu::core::Mat4 {
    fn to_bevy(self) -> bevy::prelude::Mat4 {
        use cgmath::Matrix;

        // cgmath stores as column-major, same as glam/Bevy
        let m = self.transpose(); // optional if you’re unsure about order
        bevy::prelude::Mat4::from_cols_array(&[
            m.x.x as f32,
            m.x.y as f32,
            m.x.z as f32,
            m.x.w as f32,
            m.y.x as f32,
            m.y.y as f32,
            m.y.z as f32,
            m.y.w as f32,
            m.z.x as f32,
            m.z.y as f32,
            m.z.z as f32,
            m.z.w as f32,
            m.w.x as f32,
            m.w.y as f32,
            m.w.z as f32,
            m.w.w as f32,
        ])
    }
}

impl ToBevy<bevy::prelude::Transform> for mu::core::Mat4 {
    fn to_bevy(self) -> bevy::prelude::Transform {
        Transform::from_matrix(self.to_bevy())
    }
}
