// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad core

pub mod boolean_op;
pub mod bounds;
pub mod color;
pub mod core_error;
pub mod geo2d;
pub mod geo3d;
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
use derive_more::From;
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
use strum::IntoStaticStr;
pub use triangle::*;

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

/// A µcad core geometry.
#[derive(Debug, Clone, IntoStaticStr, From)]
pub enum Geometry {
    /// Geometry 2D
    Geometry2D(Geometry2D),
    /// Geometry 3D
    Geometry3D(Geometry3D),
}

impl Geometry {
    /// Create a new 2D geometry.
    pub fn geo2d(geo2d: impl Into<Geometry2D>) -> Self {
        Self::Geometry2D(geo2d.into())
    }

    /// Create a new 3D geometry.
    pub fn geo3d(geo3d: impl Into<Geometry3D>) -> Self {
        Self::Geometry3D(geo3d.into())
    }

    /// Return geometry type.
    pub fn ty(&self) -> GeometryType {
        match self {
            Geometry::Geometry2D(_) => GeometryType::Geometry2D,
            Geometry::Geometry3D(_) => GeometryType::Geometry3D,
        }
    }
}

impl TransformAffine for Geometry {
    fn transform_affine(&mut self, m: &Mat4) {
        match self {
            Geometry::Geometry2D(geo2d) => geo2d.transform_affine(m),
            Geometry::Geometry3D(geo3d) => geo3d.transform_affine(m),
        }
    }
}

/// Type of a
#[derive(Debug, Default, Copy, Clone)]
pub enum GeometryType {
    /// An empty geometry.
    #[default]
    Empty,
    /// A 2D geometry.
    Geometry2D,
    /// A 3D geometry
    Geometry3D,
    /// A mixed geometry.
    Mixed,
}

impl std::fmt::Display for GeometryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GeometryType::Empty => "<EMPTY>",
                GeometryType::Geometry2D => "2D",
                GeometryType::Geometry3D => "3D",
                GeometryType::Mixed => "2D/3D",
            }
        )
    }
}

impl GeometryType {
    /// Merges two geometry types into a combined type.
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, x) | (x, Self::Empty) => x,
            (Self::Geometry2D, Self::Geometry2D) => Self::Geometry2D,
            (Self::Geometry3D, Self::Geometry3D) => Self::Geometry3D,
            _ => Self::Mixed,
        }
    }
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(
            f,
            "{name}({geo})",
            geo = match self {
                Self::Geometry2D(geo2d) => format!("{geo2d}"),
                Self::Geometry3D(geo3d) => format!("{geo3d}"),
            }
        )
    }
}

impl CalcBounds3D for Geometry {
    fn calc_bounds_3d(&self) -> Bounds3D {
        match self {
            Geometry::Geometry2D(geo2d) => geo2d.calc_bounds_2d().into(),
            Geometry::Geometry3D(geo3d) => geo3d.calc_bounds_3d(),
        }
    }
}
