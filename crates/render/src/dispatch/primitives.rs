use microcad_builtin::mu;
use microcad_core::{self as core, Polygon};

use crate::{RenderPrimitive, RenderResolution};

trait RenderPolygon {
    fn render_polygon(&self, resolution: &RenderResolution) -> Polygon;
}

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
