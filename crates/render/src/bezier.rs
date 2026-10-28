// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bézier curves rendered into polygons or lines, e.g. of SVG paths.

use mucad_core::{
    self as core, BooleanOps,
    boolean_op::OpType2D,
    geo2d::bezier::{Bezier, TValue},
};

use crate::RenderResolution;

/// The maximum number of line segments of a curve.
const MAX_SEGMENTS: usize = 1024;

/// The number of line segments of `curve`, which deviate from it by at most the linear
/// resolution, after Wang's formula, e.g. one for a straight line.
fn segments(curve: &Bezier, resolution: &RenderResolution) -> usize {
    let points: Vec<_> = curve.get_points().collect();
    let degree = points.len() - 1;
    // The largest second difference of the control points, which is zero for a straight line.
    let second_difference = (points.windows(3))
        .map(|p| (p[0] - 2.0 * p[1] + p[2]).length())
        .fold(0.0, f64::max);
    let squared =
        (degree * degree.saturating_sub(1)) as f64 * second_difference / (8.0 * resolution.linear);
    (squared.sqrt().ceil() as usize).clamp(1, MAX_SEGMENTS)
}

/// A point of a curve as coordinate.
fn coord(point: impl Into<[f64; 2]>) -> core::Coord {
    let [x, y] = point.into();
    core::Coord { x, y }
}

/// The points of the curves of a subpath, e.g. of a contour, as line string.
fn line_string(curves: &[Bezier], resolution: &RenderResolution) -> core::LineString {
    let mut coords = Vec::new();
    for curve in curves {
        let n = segments(curve, resolution);
        coords
            .extend((0..n).map(|i| coord(curve.evaluate(TValue::Parametric(i as f64 / n as f64)))));
    }
    if let Some(last) = curves.last() {
        coords.push(coord(last.end));
    }
    core::LineString::new(coords)
}

/// Render the subpaths of Bézier curves, e.g. of an SVG path, into a 2D geometry with the
/// linear resolution.
///
/// Closed, the subpaths are the outlines of polygons with the even-odd rule: a subpath inside
/// another one is a hole, e.g. in the letter `o`, separate subpaths are separate polygons. Open,
/// the subpaths are lines.
pub fn render_curves(
    subpaths: &[Vec<Bezier>],
    closed: bool,
    resolution: &RenderResolution,
) -> core::Geometry2D {
    let line_strings = (subpaths.iter())
        .filter(|curves| !curves.is_empty())
        .map(|curves| line_string(curves, resolution));
    match closed {
        false => {
            core::Geometry2D::MultiLineString(core::MultiLineString::new(line_strings.collect()))
        }
        true => {
            let multi_polygon = line_strings
                .map(|ring| core::MultiPolygon::new(vec![core::Polygon::new(ring, vec![])]))
                .reduce(|polygons, polygon| polygons.boolean_op(&polygon, OpType2D::Xor))
                .unwrap_or_else(|| core::MultiPolygon::new(vec![]));
            core::Geometry2D::MultiPolygon(multi_polygon)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mucad_core::svg::parse_path;
    use test_case::test_case;

    /// The geometry of the SVG path `d` with a resolution of 0.1mm.
    fn render(d: &str, closed: bool) -> core::Geometry2D {
        let subpaths = parse_path(d).expect("A path");
        render_curves(&subpaths, closed, &RenderResolution::high())
    }

    /// The polygons of a closed path.
    fn polygons(d: &str) -> Vec<core::Polygon> {
        match render(d, true) {
            core::Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.0,
            geometry => panic!("No polygons: {geometry:?}"),
        }
    }

    /// The area of a ring by the shoelace formula, without sign.
    fn ring_area(ring: &core::LineString) -> f64 {
        let area: f64 = (ring.0.windows(2))
            .map(|c| c[0].x * c[1].y - c[1].x * c[0].y)
            .sum();
        area.abs() / 2.0
    }

    /// The area of polygons without their holes.
    fn area(polygons: &[core::Polygon]) -> f64 {
        (polygons.iter())
            .map(|polygon| {
                ring_area(polygon.exterior())
                    - polygon.interiors().iter().map(ring_area).sum::<f64>()
            })
            .sum()
    }

    /// A straight line is one segment, a curve has more segments for a finer resolution.
    #[test]
    fn segments_by_resolution() {
        let line = parse_path("M 0 0 L 30 0").expect("A path")[0][0];
        assert_eq!(segments(&line, &RenderResolution::high()), 1);

        let curve = parse_path("M 0 0 C 0 10 20 10 20 0").expect("A path")[0][0];
        let coarse = segments(&curve, &RenderResolution::coarse());
        let fine = segments(&curve, &RenderResolution::high());
        assert!(coarse >= 2 && fine > 2 * coarse, "{coarse} {fine}");
    }

    /// Open subpaths are lines, which keep their start and end.
    #[test]
    fn open() {
        let core::Geometry2D::MultiLineString(lines) = render("M 0 0 L 1 0 M 5 5 L 6 5 6 6", false)
        else {
            panic!("No lines");
        };
        let coords: Vec<Vec<(f64, f64)>> = (lines.0.iter())
            .map(|line| line.0.iter().map(|c| (c.x, c.y)).collect())
            .collect();
        assert_eq!(
            coords,
            [
                vec![(0.0, 0.0), (1.0, 0.0)],
                vec![(5.0, 5.0), (6.0, 5.0), (6.0, 6.0)]
            ]
        );
    }

    /// Closed subpaths are polygons with the even-odd rule: a subpath inside another one is a
    /// hole, whichever its direction, separate subpaths are separate polygons.
    #[test_case("M 0 0 H 10 V 10 H 0 Z", 1, 0, 100.0 ; "square")]
    #[test_case("M 0 0 H 10 V 10 H 0", 1, 0, 100.0 ; "square without close")]
    #[test_case("M 0 0 H 10 V 10 H 0 Z M 2 2 H 8 V 8 H 2 Z", 1, 1, 64.0 ; "hole in the same direction")]
    #[test_case("M 0 0 H 10 V 10 H 0 Z M 2 2 V 8 H 8 V 2 Z", 1, 1, 64.0 ; "hole in the other direction")]
    #[test_case("M 0 0 H 10 V 10 H 0 Z M 20 0 H 30 V 10 H 20 Z", 2, 0, 200.0 ; "separate squares")]
    #[test_case("M 0 0 H 10 V 10 H 0 Z M 5 0 H 15 V 10 H 5 Z", 2, 0, 100.0 ; "overlap is a hole")]
    fn closed(d: &str, polygon_count: usize, hole_count: usize, expected_area: f64) {
        let polygons = polygons(d);
        assert_eq!(polygons.len(), polygon_count);
        let holes: usize = polygons
            .iter()
            .map(|polygon| polygon.interiors().len())
            .sum();
        assert_eq!(holes, hole_count);
        assert!(
            (area(&polygons) - expected_area).abs() < 1e-9,
            "{}",
            area(&polygons)
        );
    }

    /// A circle of arcs is a polygon, whose points are on the circle and whose edges deviate from
    /// it by at most the resolution of 0.1mm.
    #[test]
    fn circle() {
        let polygons = polygons("M -5 0 A 5 5 0 1 1 5 0 A 5 5 0 1 1 -5 0 Z");
        assert_eq!(polygons.len(), 1);
        let radius = |c: core::Coord| (c.x * c.x + c.y * c.y).sqrt();
        let exterior = polygons[0].exterior();
        for c in &exterior.0 {
            assert!((radius(*c) - 5.0).abs() < 0.01, "radius {}", radius(*c));
        }
        // The middle of an edge is its farthest point from the circle.
        for edge in exterior.0.windows(2) {
            let middle = (edge[0] + edge[1]) / 2.0;
            assert!(
                5.0 - radius(middle) <= 0.1,
                "deviation {}",
                5.0 - radius(middle)
            );
        }
    }

    /// Without subpaths, there are no polygons and no lines.
    #[test]
    fn empty() {
        assert!(polygons("").is_empty());
        let core::Geometry2D::MultiLineString(lines) = render("M 1 1", false) else {
            panic!("No lines");
        };
        assert!(lines.0.is_empty());
    }
}
