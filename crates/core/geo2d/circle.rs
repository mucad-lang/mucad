// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! 2D Geometry

use geo::Translate;

use crate::*;

/// Circle with offset.
#[derive(Debug, Clone)]
pub struct Circle {
    /// Radius of the circle.
    pub radius: Scalar,

    /// Offset.
    pub offset: Vec2,
}

impl CalcBounds2D for Circle {
    fn calc_bounds_2d(&self) -> Bounds2D {
        use geo::Coord;

        if self.radius > 0.0 {
            let r = Vec2::new(self.radius, self.radius);
            let min: (Scalar, Scalar) = (self.offset - r).into();
            let max: (Scalar, Scalar) = (self.offset + r).into();

            Some(Rect::new(Coord::from(min), Coord::from(max)))
        } else {
            None
        }
        .into()
    }
}

impl FetchPoints2D for Circle {
    fn fetch_points_2d(&self) -> Vec<Vec2> {
        vec![self.offset]
    }
}

impl Circle {
    /// Render a circle with a radius into a polygon.
    pub fn circle_polygon(radius: Scalar, n: u32) -> Polygon {
        let points = NgonIterator::new(n).map(|p| p * radius).collect();
        Polygon::new(LineString::new(points), vec![])
    }

    /// Convert the circle to polygon.
    pub fn to_polygon(&self, n: u32) -> Polygon {
        let polygon = Self::circle_polygon(self.radius, n);
        polygon.translate(self.offset.x, self.offset.y)
    }
}
