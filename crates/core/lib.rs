// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad core

pub mod boolean_op;
pub mod bounds;
pub mod color;
pub mod core_error;
pub mod geo2d;
pub mod geo3d;
pub mod geometry;
pub mod length;
pub mod traits;
pub mod triangle;

/// Primitive integer type.
pub type Integer = i64;
/// Primitive floating point type.
pub type Scalar = f64;
/// 2D vector type.
pub type Vec2 = cgmath::Vector2<Scalar>;
/// 3D vector type.
pub type Vec3 = cgmath::Vector3<Scalar>;
/// 4D vector type.
pub type Vec4 = cgmath::Vector4<Scalar>;
/// 2D matrix type.
pub type Mat2 = cgmath::Matrix2<Scalar>;
/// 3D matrix type.
pub type Mat3 = cgmath::Matrix3<Scalar>;
/// 4D matrix type.
pub type Mat4 = cgmath::Matrix4<Scalar>;
/// Primitive angle type in radians.
pub type Angle = cgmath::Rad<Scalar>;
/// Length type.
pub use length::Length;

/// Constants.
pub mod consts {
    pub use std::f64::consts::PI;
    pub use std::f64::consts::TAU;
}

pub use boolean_op::{BooleanOp, UnaryBooleanOp};

pub use bounds::*;
pub use color::*;
pub use core_error::*;
pub use geo2d::*;
pub use geo3d::*;
pub use geometry::{Geometry, GeometryType};
pub use triangle::Triangle;

use crate::traits::TransformAffine;

/// Convert a Matrix4 to Matrix3.
pub fn mat4_to_mat3(m: &Mat4) -> Mat3 {
    Mat3::from_cols(m.x.truncate_n(2), m.y.truncate_n(2), m.w.truncate_n(2))
}

/// Convert a Matrix3 to Matrix4.
pub fn mat3_to_mat4(m: &Mat3) -> Mat4 {
    Mat4::new(
        m.x.x, m.x.y, 0.0, m.x.z, // First column: X basis + X translation
        m.y.x, m.y.y, 0.0, m.y.z, // Second column: Y basis + Y translation
        0.0, 0.0, 1.0, 0.0, // Z axis: identity (no change)
        0.0, 0.0, 0.0, 1.0, // Homogeneous row
    )
}
