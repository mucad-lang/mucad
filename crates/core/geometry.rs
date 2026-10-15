use derive_more::From;
use strum::IntoStaticStr;

use crate::{Bounds3D, CalcBounds2D, CalcBounds3D, Geometry2D, Geometry3D, Mat4, traits};

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

impl traits::Center for Geometry {
    fn center(&self) -> Self {
        match self {
            Self::Geometry2D(geo2d) => geo2d.center().into(),
            Self::Geometry3D(geo3d) => geo3d.center().into(),
        }
    }
}

impl traits::TransformAffine for Geometry {
    fn transform_affine(&mut self, m: &Mat4) {
        match self {
            Self::Geometry2D(geo2d) => geo2d.transform_affine(m),
            Self::Geometry3D(geo3d) => geo3d.transform_affine(m),
        }
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

/// Type of a Geometry
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
