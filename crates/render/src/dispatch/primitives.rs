// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::mu as __mu;
use mucad_lang_types::math::IntoFloat;

use crate::{
    GeometryOutputs, GeometryTree, Render, RenderContext, RenderError, RenderPrimitive,
    RenderResolution, bezier,
};

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

/// The outline of an involute spur gear.
/// - the addendum is the module, the dedendum the module and the clearance,
/// - the teeth are as thick as the gaps at the pitch circle, half the circular pitch, less the
///   backlash,
/// - the flanks are involutes of the base circle and, below it, radial lines down to the root
///   circle, the gaps between the teeth are arcs of the root circle.
///
/// A gear without at least three teeth, a size, a pressure angle in `0°..90°`, a root circle or
/// teeth, e.g. with a backlash of the circular pitch, is empty.
impl RenderPolygon for __mu::geo2d::InvoluteGearProfile {
    fn render_polygon(&self, resolution: &RenderResolution) -> mucad_core::Polygon {
        use mucad_core::{Coord, LineString, Polygon, Scalar, consts::PI};

        let teeth: i64 = self.teeth.to_num();
        let module = self.module.as_mm();
        let pressure_angle: mucad_core::Angle = self.pressure_angle.into_float();
        let empty = || Polygon::new(LineString::new(vec![]), vec![]);
        if teeth < 3 || module <= 0.0 || !(0.0..PI / 2.0).contains(&pressure_angle.0) {
            return empty();
        }

        let z = teeth as Scalar;
        let pitch_radius = module * z / 2.0;
        let base_radius = pitch_radius * pressure_angle.0.cos();
        let outer_radius = pitch_radius + module;
        let root_radius = pitch_radius - module - self.clearance.as_mm();
        // Half a tooth at the pitch circle is a quarter of the angle between two teeth, less
        // the backlash: `(360° / teeth - backlash_angle) / 4` in OpenSCAD.
        let half_tooth = (2.0 * PI / z - self.backlash.as_mm() / pitch_radius) / 4.0;
        if root_radius <= 0.0 || half_tooth <= 0.0 {
            return empty();
        }
        // The involutes start at the base circle, or at the root circle above it.
        let start_radius = base_radius.max(root_radius);

        // The parameter of the involute of the base circle at a radius.
        let involute_parameter =
            |radius: Scalar| ((radius / base_radius).powi(2) - 1.0).max(0.0).sqrt();
        // The polar angle of the involute at its parameter, e.g. `tan φ - φ` at the pitch circle.
        let involute_angle = |t: Scalar| t - t.atan();
        let pitch_angle = involute_angle(involute_parameter(pitch_radius));
        // The angle between a flank and the center of its tooth at the involute parameter `t`,
        // zero where the flanks of a pointed tooth meet.
        let flank_angle = |t: Scalar| (half_tooth - involute_angle(t) + pitch_angle).max(0.0);
        let point = |radius: Scalar, angle: Scalar| {
            Coord::from((radius * angle.cos(), radius * angle.sin()))
        };

        // The involute parameters of a flank from its start to the tip, evenly like in OpenSCAD.
        let facets = (resolution.circular_segments(outer_radius) as Scalar / z)
            .ceil()
            .max(4.0) as usize;
        let (t_start, t_stop) = (
            involute_parameter(start_radius),
            involute_parameter(outer_radius),
        );
        let flank: Vec<(Scalar, Scalar)> = (0..=facets)
            .map(|i| {
                let t = t_start + (t_stop - t_start) * i as Scalar / facets as Scalar;
                (base_radius * (1.0 + t * t).sqrt(), flank_angle(t))
            })
            .collect();
        let below_base = root_radius < base_radius;
        let start_angle = flank_angle(t_start);

        // The arc of the root circle between two teeth, without its ends.
        let gap_angle = 2.0 * PI / z - 2.0 * start_angle;
        let gap_segments = (resolution.circular_segments(root_radius) as Scalar * gap_angle
            / (2.0 * PI))
            .ceil()
            .max(1.0) as usize;

        let mut points = Vec::new();
        for tooth in 0..teeth {
            let center = tooth as Scalar * 2.0 * PI / z;
            // The flank before the center up to the tip, then the flank after it down to the root.
            if below_base {
                points.push(point(root_radius, center - start_angle));
            }
            points.extend(flank.iter().map(|(r, a)| point(*r, center - a)));
            points.extend(flank.iter().rev().map(|(r, a)| point(*r, center + a)));
            if below_base {
                points.push(point(root_radius, center + start_angle));
            }
            if gap_angle > 0.0 {
                points.extend((1..gap_segments).map(|i| {
                    let angle =
                        center + start_angle + gap_angle * i as Scalar / gap_segments as Scalar;
                    point(root_radius, angle)
                }));
            }
        }
        Polygon::new(LineString::new(points), vec![])
    }
}

/// An SVG path is parsed into Bézier curves, which are rendered into polygons or lines.
///
/// An invalid path is reported as error and rendered as nothing, so that the other models are
/// rendered.
impl Render for __mu::geo2d::SvgPath {
    fn render(&self, tree: &GeometryTree, context: &mut RenderContext) -> crate::RenderResult {
        let mut subpaths = match mucad_core::svg::parse_path(&self.d) {
            Ok(subpaths) => subpaths,
            Err(err) => {
                let src_ref = (context.model(tree).creator.as_ref())
                    .map(|creator| creator.src_ref)
                    .unwrap_or_else(mucad_lang_base::SrcRef::none);
                context.push_error(RenderError::InvalidSvgPath {
                    message: err.to_string(),
                    src_ref,
                });
                return Ok(GeometryOutputs::default());
            }
        };
        // The y axis points down in SVG and up in µcad.
        if self.flip {
            for curves in &mut subpaths {
                for curve in curves {
                    *curve = curve.apply_transformation(|point| point.with_y(-point.y));
                }
            }
        }
        let resolution = context.current_resolution(tree);
        let geometry = bezier::render_curves(&subpaths, self.closed, &resolution);
        Ok(mucad_core::Geometry::geo2d(geometry).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mucad_core::{Polygon, Scalar, consts::PI};

    /// An involute gear profile with the module in mm and the pressure angle in degrees,
    /// without backlash and with the clearance of a quarter module, like in OpenSCAD.
    fn gear(module: Scalar, teeth: i64, pressure_angle: Scalar) -> Polygon {
        gear_with(module, teeth, pressure_angle, 0.0, module / 4.0)
    }

    /// An involute gear profile with a backlash and a clearance in mm.
    fn gear_with(
        module: Scalar,
        teeth: i64,
        pressure_angle: Scalar,
        backlash: Scalar,
        clearance: Scalar,
    ) -> Polygon {
        use mucad_lang_types::Length;
        __mu::geo2d::InvoluteGearProfile {
            module: Length::mm(module),
            teeth: mucad_lang_types::Integer::from_num(teeth),
            pressure_angle: cgmath::Rad(mucad_lang_types::Scalar::from_num(
                pressure_angle.to_radians(),
            )),
            backlash: Length::mm(backlash),
            clearance: Length::mm(clearance),
        }
        .render_polygon(&RenderResolution::high())
    }

    /// The part of the pitch circle inside the gear.
    fn pitch_circle_inside(polygon: &Polygon, pitch_radius: Scalar) -> Scalar {
        let samples = 100_000;
        let inside = (0..samples)
            .filter(|i| {
                let angle = 2.0 * PI * (*i as Scalar + 0.5) / samples as Scalar;
                contains(
                    polygon,
                    (pitch_radius * angle.cos(), pitch_radius * angle.sin()),
                )
            })
            .count();
        inside as Scalar / samples as Scalar
    }

    /// The smallest distance of a point of the outline to the center.
    fn min_radius(polygon: &Polygon) -> Scalar {
        polygon
            .exterior()
            .coords()
            .map(|p| p.x.hypot(p.y))
            .fold(Scalar::MAX, Scalar::min)
    }

    /// Whether `(x, y)` is inside `polygon`, by counting the crossings of a ray to the right.
    fn contains(polygon: &Polygon, (x, y): (Scalar, Scalar)) -> bool {
        let points: Vec<_> = polygon.exterior().coords().collect();
        let mut inside = false;
        for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
            if (a.y > y) != (b.y > y) && x < a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x) {
                inside = !inside;
            }
        }
        inside
    }

    /// The teeth are as thick as the gaps at the pitch circle: half of it is inside the gear.
    #[test_case::test_case(2.0, 20, 20.0; "20 teeth")]
    #[test_case::test_case(1.0, 12, 20.0; "12 teeth with radial flanks")]
    #[test_case::test_case(1.5, 60, 25.0; "60 teeth starting at the root circle")]
    fn tooth_thickness_at_pitch_circle(module: Scalar, teeth: i64, pressure_angle: Scalar) {
        let polygon = gear(module, teeth, pressure_angle);
        let ratio = pitch_circle_inside(&polygon, module * teeth as Scalar / 2.0);
        assert!((ratio - 0.5).abs() < 0.005, "{ratio}");
    }

    /// The outline is between the root and the tip circle and goes once around the center
    /// without turning back, so it does not cross itself.
    #[test_case::test_case(1.0, 8, 20.0; "8 teeth")]
    #[test_case::test_case(2.0, 20, 20.0; "20 teeth")]
    #[test_case::test_case(1.0, 100, 20.0; "100 teeth")]
    #[test_case::test_case(3.0, 6, 30.0; "pointed teeth")]
    fn outline(module: Scalar, teeth: i64, pressure_angle: Scalar) {
        let polygon = gear(module, teeth, pressure_angle);
        let pitch_radius = module * teeth as Scalar / 2.0;
        let (root_radius, outer_radius) = (pitch_radius - 1.25 * module, pitch_radius + module);
        let eps = 1e-9;

        let points: Vec<_> = polygon.exterior().coords().collect();
        let radii: Vec<Scalar> = points.iter().map(|p| p.x.hypot(p.y)).collect();
        let max = radii.iter().cloned().fold(Scalar::MIN, Scalar::max);
        let min = radii.iter().cloned().fold(Scalar::MAX, Scalar::min);
        assert!((max - outer_radius).abs() < eps, "{max} != {outer_radius}");
        assert!((min - root_radius).abs() < eps, "{min} != {root_radius}");

        let mut turned = 0.0;
        for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
            let mut step = b.y.atan2(b.x) - a.y.atan2(a.x);
            if step < -PI {
                step += 2.0 * PI;
            }
            assert!(step > -eps, "Turns back by {step}");
            turned += step;
        }
        assert!((turned - 2.0 * PI).abs() < eps, "{turned}");
    }

    /// The backlash makes the teeth thinner at the pitch circle: by `backlash / (2π module)` of
    /// it, the circular pitch is `π module`.
    #[test_case::test_case(2.0, 20, 0.5; "half a millimeter")]
    #[test_case::test_case(1.0, 12, 0.2; "12 teeth")]
    fn backlash(module: Scalar, teeth: i64, backlash: Scalar) {
        let polygon = gear_with(module, teeth, 20.0, backlash, module / 4.0);
        let ratio = pitch_circle_inside(&polygon, module * teeth as Scalar / 2.0);
        let expected = 0.5 - backlash / (2.0 * PI * module);
        assert!((ratio - expected).abs() < 0.005, "{ratio} != {expected}");
    }

    /// The clearance deepens the root circle below the tips of the other gear.
    #[test_case::test_case(0.0; "no clearance")]
    #[test_case::test_case(0.5; "half a module")]
    #[test_case::test_case(1.2; "more than the module")]
    fn clearance(clearance: Scalar) {
        let (module, teeth) = (2.0, 30);
        let polygon = gear_with(module, teeth, 20.0, 0.0, clearance);
        let root_radius = module * teeth as Scalar / 2.0 - module - clearance;
        let min = min_radius(&polygon);
        assert!((min - root_radius).abs() < 1e-9, "{min} != {root_radius}");
    }

    /// A gear without three teeth, a size or a valid pressure angle is empty.
    #[test_case::test_case(1.0, 2, 20.0; "two teeth")]
    #[test_case::test_case(0.0, 20, 20.0; "no module")]
    #[test_case::test_case(1.0, 20, 90.0; "no base circle")]
    fn invalid(module: Scalar, teeth: i64, pressure_angle: Scalar) {
        assert!(gear(module, teeth, pressure_angle).exterior().0.is_empty());
    }

    /// A gear without teeth or root circle is empty: with a backlash of the circular pitch or
    /// a clearance, which is deeper than the center.
    #[test_case::test_case(PI * 2.0, 0.5; "backlash of the circular pitch")]
    #[test_case::test_case(0.0, 20.0; "no root circle")]
    fn no_teeth_or_root(backlash: Scalar, clearance: Scalar) {
        assert!(
            gear_with(2.0, 12, 20.0, backlash, clearance)
                .exterior()
                .0
                .is_empty()
        );
    }
}
