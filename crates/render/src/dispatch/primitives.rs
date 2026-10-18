// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::mu;
use mucad_core::{self as core, Polygon};

use crate::{RenderPrimitive, RenderResolution};

/// Helper trait to render a polygon.
trait RenderPolygon {
    fn render_polygon(&self, resolution: &RenderResolution) -> Polygon;
}

/// Everything that can be rendered into a polygon, can be rendered as a primitive.
impl<T> RenderPrimitive for T
where
    T: RenderPolygon,
{
    fn render_primitive(&self, resolution: &RenderResolution) -> core::Geometry {
        let polygon = self.render_polygon(resolution);
        core::Geometry2D::Polygon(polygon).into()
    }
}

impl RenderPolygon for mu::geo2d::Circle {
    fn render_polygon(&self, resolution: &RenderResolution) -> Polygon {
        let radius = self.radius.as_mm();
        let n = resolution.circular_segments(radius);
        core::Circle::circle_polygon(radius, n)
    }
}

impl RenderPolygon for mu::geo2d::Rect {
    fn render_polygon(&self, _: &RenderResolution) -> Polygon {
        let (x, y, w, h) = (
            self.x.as_mm(),
            self.y.as_mm(),
            self.width.as_mm(),
            self.height.as_mm(),
        );
        core::Rect::new((x, y), (x + w, y + h)).to_polygon()
    }
}

impl RenderPrimitive for mu::geo3d::Sphere {
    fn render_primitive(&self, resolution: &RenderResolution) -> core::Geometry {
        let radius = self.radius.as_mm();

        core::Geometry3D::from(core::geo3d::Manifold::sphere(
            radius,
            resolution.circular_segments(radius) as i32,
        ))
        .into()
    }
}
