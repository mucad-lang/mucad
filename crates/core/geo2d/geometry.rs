// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::traits::{Center, TotalMemory, VertexCount};
use derive_more::From;

use geo::MultiPolygon;
use strum::IntoStaticStr;

/// A 2D Geometry which is independent from resolution.
///
/// A 2D Geometry is closed under arbitrary 2D affine transformations.
#[derive(IntoStaticStr, From, Clone, Debug)]
pub enum Geometry2D {
    /// Line string.
    LineString(LineString),
    /// Multiple line strings.
    MultiLineString(MultiLineString),
    /// Polygon.
    Polygon(Polygon),
    /// Multiple polygons.
    MultiPolygon(MultiPolygon),
    /// Line.
    Line(Line),
    /// Collection,
    Collection(Geometries2D),
}

impl Geometry2D {
    /// Return name of geometry.
    pub fn name(&self) -> &'static str {
        self.into()
    }

    /// Apply boolean operation.
    pub fn boolean_op(self, other: Self, op: BooleanOp) -> geo2d::MultiPolygon {
        use geo::BooleanOps;
        let lhs: MultiPolygon = self.into();
        let rhs: MultiPolygon = other.into();
        lhs.boolean_op(&rhs, op.into())
    }

    /// Returns true if this geometry fills an area (e.g. like a polygon or circle).
    pub fn is_areal(&self) -> bool {
        !matches!(
            self,
            Geometry2D::LineString(_)
                | Geometry2D::MultiLineString(_)
                | Geometry2D::Line(_)
                | Geometry2D::Collection(_)
        )
    }

    /// Return this geometry with calculated bounds.
    pub fn with_bounds(self) -> WithBounds2D<Geometry2D> {
        let bounds = self.calc_bounds_2d();
        WithBounds2D {
            bounds,
            inner: self,
        }
    }
}

impl std::fmt::Display for Geometry2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl CalcBounds2D for MultiPolygon {
    fn calc_bounds_2d(&self) -> Bounds2D {
        use geo::BoundingRect;
        self.bounding_rect().into()
    }
}

impl CalcBounds2D for Geometry2D {
    fn calc_bounds_2d(&self) -> Bounds2D {
        use geo::BoundingRect;

        match &self {
            Geometry2D::LineString(line_string) => line_string.bounding_rect().into(),
            Geometry2D::MultiLineString(multi_line_string) => {
                multi_line_string.bounding_rect().into()
            }
            Geometry2D::Polygon(polygon) => polygon.bounding_rect().into(),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.calc_bounds_2d(),
            Geometry2D::Line(line) => line.calc_bounds_2d(),
            Geometry2D::Collection(collection) => collection.calc_bounds_2d(),
        }
    }
}

impl Transformed2D for Geometry2D {
    fn transformed_2d(&self, mat: &Mat3) -> Self {
        match self {
            Geometry2D::LineString(line_string) => line_string.transformed_2d(mat).into(),
            Geometry2D::MultiLineString(mls) => mls.transformed_2d(mat).into(),
            Geometry2D::Polygon(polygon) => polygon.transformed_2d(mat).into(),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.transformed_2d(mat).into(),
            Geometry2D::Line(line) => line.transformed_2d(mat).into(),
            Geometry2D::Collection(geometries) => geometries.transformed_2d(mat).into(),
        }
    }
}

impl TransformAffine for Geometry2D {
    fn transform_affine(&mut self, m: &Mat4) {
        *self = self.transformed_2d(&mat4_to_mat3(m)); // TODO Optimize this
    }
}

impl Center for Geometry2D {
    fn center(&self) -> Self {
        if let Some(bounds) = self.calc_bounds_2d().rect() {
            let d: Vec2 = bounds.center().x_y().into();
            self.transformed_2d(&Mat3::from_translation(-d))
        } else {
            self.clone()
        }
    }
}

impl geo::Buffer for Geometry2D {
    type Scalar = Scalar;

    fn buffer_with_style(
        &self,
        style: geo::buffer::BufferStyle<Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        match &self {
            Geometry2D::LineString(line_string) => line_string.buffer_with_style(style),
            Geometry2D::MultiLineString(multi_line_string) => {
                multi_line_string.buffer_with_style(style)
            }
            Geometry2D::Polygon(polygon) => polygon.buffer_with_style(style),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.buffer_with_style(style),
            Geometry2D::Line(line) => {
                LineString::new(vec![line.0.into(), line.1.into()]).buffer_with_style(style)
            }
            Geometry2D::Collection(collection) => collection.buffer_with_style(style),
        }
    }
}

impl From<Geometry2D> for MultiPolygon {
    fn from(geo: Geometry2D) -> Self {
        match geo {
            Geometry2D::Polygon(polygon) => polygon.into(),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon,
            Geometry2D::Collection(collection) => collection.into(),
            _ => MultiPolygon::empty(),
        }
    }
}

impl TotalMemory for LineString {
    fn heap_memory(&self) -> usize {
        self.0.heap_memory()
    }
}

impl TotalMemory for MultiLineString {
    fn heap_memory(&self) -> usize {
        self.0.iter().map(|l| l.heap_memory()).sum()
    }
}

impl TotalMemory for Polygon {
    fn heap_memory(&self) -> usize {
        self.exterior().heap_memory()
            + self
                .interiors()
                .iter()
                .map(|l| l.heap_memory())
                .sum::<usize>()
    }
}

impl TotalMemory for MultiPolygon {
    fn heap_memory(&self) -> usize {
        self.0.iter().map(|p| p.heap_memory()).sum()
    }
}

impl TotalMemory for Rect {}
impl TotalMemory for Line {}

impl TotalMemory for Geometry2D {
    fn heap_memory(&self) -> usize {
        match &self {
            Geometry2D::LineString(line_string) => line_string.heap_memory(),
            Geometry2D::MultiLineString(multi_line_string) => multi_line_string.heap_memory(),
            Geometry2D::Polygon(polygon) => polygon.heap_memory(),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.heap_memory(),
            Geometry2D::Line(line) => line.heap_memory(),
            Geometry2D::Collection(collection) => collection.heap_memory(),
        }
    }
}

impl VertexCount for LineString {
    fn vertex_count(&self) -> usize {
        self.0.len()
    }
}

impl VertexCount for MultiLineString {
    fn vertex_count(&self) -> usize {
        self.iter().map(|l| l.vertex_count()).sum()
    }
}

impl VertexCount for Polygon {
    fn vertex_count(&self) -> usize {
        self.exterior().vertex_count()
            + self
                .interiors()
                .iter()
                .map(|l| l.vertex_count())
                .sum::<usize>()
    }
}

impl VertexCount for MultiPolygon {
    fn vertex_count(&self) -> usize {
        self.iter().map(|p| p.vertex_count()).sum()
    }
}

impl VertexCount for Rect {
    fn vertex_count(&self) -> usize {
        4
    }
}

impl VertexCount for Line {
    fn vertex_count(&self) -> usize {
        2
    }
}

impl VertexCount for Geometry2D {
    fn vertex_count(&self) -> usize {
        match &self {
            Geometry2D::LineString(line_string) => line_string.vertex_count(),
            Geometry2D::MultiLineString(multi_line_string) => multi_line_string.vertex_count(),
            Geometry2D::Polygon(polygon) => polygon.vertex_count(),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.vertex_count(),
            Geometry2D::Line(line) => line.vertex_count(),
            Geometry2D::Collection(collection) => collection.vertex_count(),
        }
    }
}
