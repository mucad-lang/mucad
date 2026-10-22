// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::mu as __mu;
use mucad_lang_types::math::IntoFloat;

use crate::{RenderPrimitive, RenderResolution};

/// Helper trait to render a polygon.
trait RenderPolygon {
    fn render_polygon(&self, resolution: &RenderResolution) -> mucad_core::Polygon;
}

/// Everything that can be rendered into a polygon, can be rendered as a primitive.
impl<T> RenderPrimitive for T
where
    T: RenderPolygon,
{
    fn render_primitive(&self, resolution: &RenderResolution) -> mucad_core::Geometry {
        let polygon = self.render_polygon(resolution);
        mucad_core::Geometry::geo2d(polygon)
    }
}

impl RenderPolygon for __mu::geo2d::Circle {
    fn render_polygon(&self, resolution: &RenderResolution) -> mucad_core::Polygon {
        let radius = self.radius.as_mm();
        let n = resolution.circular_segments(radius);
        mucad_core::Circle::circle_polygon(radius, n)
    }
}

impl RenderPolygon for __mu::geo2d::Rect {
    fn render_polygon(&self, _: &RenderResolution) -> mucad_core::Polygon {
        let (x, y, w, h) = (
            self.x.as_mm(),
            self.y.as_mm(),
            self.width.as_mm(),
            self.height.as_mm(),
        );
        mucad_core::Rect::new((x, y), (x + w, y + h)).to_polygon()
    }
}

impl RenderPrimitive for __mu::geo3d::Sphere {
    fn render_primitive(&self, resolution: &RenderResolution) -> mucad_core::Geometry {
        let radius = self.radius.as_mm();

        mucad_core::Geometry::geo3d(mucad_core::Manifold::sphere(
            radius,
            resolution.circular_segments(radius) as i32,
        ))
    }
}

struct Sector {
    /// Radius of the circle.
    pub radius: mucad_core::Scalar,
    /// Start angle.
    pub start: mucad_core::Angle,
    /// End angle.
    pub end: mucad_core::Angle,
}

impl Sector {
    /// A sector is a full circle when `offset_angle >= 360°`.
    pub fn is_circle(&self) -> bool {
        self.offset_angle() >= cgmath::Deg(360.0).into()
    }

    /// Calculate offset angle.
    pub fn offset_angle(&self) -> mucad_core::Angle {
        cgmath::Rad((self.end - self.start).0.abs())
    }
}

impl RenderPolygon for __mu::geo2d::Sector {
    fn render_polygon(&self, resolution: &RenderResolution) -> mucad_core::Polygon {
        let sector = Sector {
            radius: self.radius.as_mm(),
            start: self.start.into_float(),
            end: self.end.into_float(),
        };
        let Sector { radius: r, .. } = &sector;

        use std::f64::consts::PI;

        let offset = sector.offset_angle();
        let n = (resolution.circular_segments(*r) as f64 * (offset.0 / PI / 2.0)) as u32;

        use mucad_core::{Coord, Scalar};
        let points: Vec<Coord> = if !sector.is_circle() && n > 0 {
            (0..=n)
                .map(|i| {
                    let angle = sector.start + offset * (i as Scalar) / (n as Scalar);
                    (angle.0.cos(), angle.0.sin())
                })
                .chain(
                    // Add center point.
                    [(0.0 as Scalar, 0.0 as Scalar)].iter().cloned(),
                )
                .map(|c| Coord::from(c) * *r)
                .collect()
        } else {
            (0..n)
                .map(|i| {
                    let angle = 2.0 * PI * (i as Scalar) / (n as Scalar);
                    (angle.cos(), angle.sin())
                })
                .map(|c| Coord::from(c) * *r)
                .collect()
        };
        mucad_core::Polygon::new(mucad_core::LineString::from(points), vec![])
    }
}

impl RenderPolygon for __mu::geo2d::SinusoidalGearProfile {
    fn render_polygon(&self, resolution: &RenderResolution) -> mucad_core::Polygon {
        use mucad_core::Scalar;
        let a = self.module.as_mm() * 2.0;
        let b = self.roundness.into_float();
        let teeth: i64 = self.teeth.to_num();
        let radius = a * teeth as Scalar * 0.5;
        let n = resolution.circular_segments(radius) * 2;
        let stretch: Scalar = self.stretch.into_float();

        mucad_core::Polygon::new(
            mucad_core::LineString::new(
                (0..n)
                    .map(|i| {
                        let theta = (i as Scalar) / (n as Scalar) * mucad_core::consts::PI * 2.0;
                        let phi_sin = (theta * (teeth as Scalar)).sin();
                        let l = a * (phi_sin.abs().powf(b) * phi_sin.signum()) * stretch + radius;

                        mucad_core::geo2d::Coord::from((l * theta.cos(), l * theta.sin()))
                    })
                    .collect(),
            ),
            vec![],
        )
    }
}
