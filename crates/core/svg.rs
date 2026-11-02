// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! SVG paths, e.g. of the `d` attribute of an SVG `<path>`, as cubic Bézier curves.

use bezier_rs::Bezier;
use svgtypes::{SimplePathSegment, SimplifyingPathParser};

use crate::Vec2;

/// An error of parsing an SVG path.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SvgError {
    /// The path data is invalid, e.g. a command without its numbers.
    #[error("invalid SVG path: {0}")]
    InvalidPath(String),
}

/// The cubic Bézier curves of the subpaths of an SVG path, e.g. the contours of a letter.
///
/// A subpath starts with a move, e.g. `M 0 0`. A closed subpath (`Z`) ends with a curve back to
/// its start, if its last point is not its start.
pub type SubpathCurves = Vec<Vec<Bezier>>;

/// A cubic Bézier curve from `start` to `end` with two handles.
fn cubic(start: Vec2, handle_start: Vec2, handle_end: Vec2, end: Vec2) -> Bezier {
    Bezier::from_cubic_coordinates(
        start.x,
        start.y,
        handle_start.x,
        handle_start.y,
        handle_end.x,
        handle_end.y,
        end.x,
        end.y,
    )
}

/// A line from `start` to `end` as cubic Bézier curve, with handles at a third of the line.
fn cubic_line(start: Vec2, end: Vec2) -> Bezier {
    let third = (end - start) / 3.0;
    cubic(start, start + third, end - third, end)
}

/// A quadratic Bézier curve as cubic Bézier curve, which is the same curve.
fn cubic_quadratic(start: Vec2, handle: Vec2, end: Vec2) -> Bezier {
    let two_thirds = |from: Vec2| from + (handle - from) * 2.0 / 3.0;
    cubic(start, two_thirds(start), two_thirds(end), end)
}

/// Parse the path data `d` of an SVG path, e.g. `M 0 0 L 10 0 L 10 10 Z`, into the cubic Bézier
/// curves of its subpaths, see [`SubpathCurves`].
///
/// All commands of SVG paths are supported, also relative ones (e.g. `l 10 0`): lines and
/// quadratic curves become cubic curves, arcs become cubic curves, which approximate them.
/// Subpaths without curves, e.g. a single move, are omitted.
pub fn parse_path(d: &str) -> Result<SubpathCurves, SvgError> {
    let mut subpaths = SubpathCurves::new();
    let mut curves = Vec::new();
    // The current point and the start of the current subpath.
    let mut current = Vec2::new(0.0, 0.0);
    let mut start = current;

    for segment in SimplifyingPathParser::from(d) {
        let segment = segment.map_err(|err| SvgError::InvalidPath(err.to_string()))?;
        match segment {
            SimplePathSegment::MoveTo { x, y } => {
                if !curves.is_empty() {
                    subpaths.push(std::mem::take(&mut curves));
                }
                current = Vec2::new(x, y);
                start = current;
            }
            SimplePathSegment::LineTo { x, y } => {
                let end = Vec2::new(x, y);
                curves.push(cubic_line(current, end));
                current = end;
            }
            SimplePathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let end = Vec2::new(x, y);
                curves.push(cubic(current, Vec2::new(x1, y1), Vec2::new(x2, y2), end));
                current = end;
            }
            SimplePathSegment::Quadratic { x1, y1, x, y } => {
                let end = Vec2::new(x, y);
                curves.push(cubic_quadratic(current, Vec2::new(x1, y1), end));
                current = end;
            }
            SimplePathSegment::ClosePath => {
                if current != start {
                    curves.push(cubic_line(current, start));
                }
                if !curves.is_empty() {
                    subpaths.push(std::mem::take(&mut curves));
                }
                current = start;
            }
        }
    }
    if !curves.is_empty() {
        subpaths.push(curves);
    }
    Ok(subpaths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezier_rs::{BezierHandles, TValue};
    use test_case::test_case;

    /// A point of a curve.
    type Point = [f64; 2];

    /// The start, the handles and the end of a cubic curve.
    fn cubic_points(curve: &Bezier) -> Vec<Point> {
        assert!(
            matches!(curve.handles, BezierHandles::Cubic { .. }),
            "No cubic curve: {curve:?}"
        );
        curve.get_points().map(Point::from).collect()
    }

    /// The distance between two points.
    fn distance(a: Point, b: Point) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// The start and end points of the curves of each subpath of `d`.
    fn points(d: &str) -> Vec<Vec<(f64, f64)>> {
        let subpaths = parse_path(d).expect("A path");
        (subpaths.iter())
            .map(|curves| {
                let mut points: Vec<_> = curves.iter().map(|c| (c.start.x, c.start.y)).collect();
                if let Some(last) = curves.last() {
                    points.push((last.end.x, last.end.y));
                }
                points
            })
            .collect()
    }

    #[test_case("M 0 0 L 10 0 L 10 10 Z", vec![vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 0.0)]] ; "closed triangle")]
    #[test_case("m 0 0 l 10 0 v 10 z", vec![vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 0.0)]] ; "relative commands")]
    #[test_case("M0,0L10-5", vec![vec![(0.0, 0.0), (10.0, -5.0)]] ; "compact numbers")]
    #[test_case("M 0 0 H 10 V 10", vec![vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]] ; "horizontal and vertical lines")]
    #[test_case("M 0 0 L 10 0 L 0 0 Z", vec![vec![(0.0, 0.0), (10.0, 0.0), (0.0, 0.0)]] ; "closed at its start")]
    #[test_case("M 0 0 L 1 0 M 5 5 L 6 5", vec![vec![(0.0, 0.0), (1.0, 0.0)], vec![(5.0, 5.0), (6.0, 5.0)]] ; "two subpaths")]
    #[test_case("M 0 0 L 1 0 Z L 0 1", vec![vec![(0.0, 0.0), (1.0, 0.0), (0.0, 0.0)], vec![(0.0, 0.0), (0.0, 1.0)]] ; "line after close starts at the start")]
    #[test_case("M 0 0 L 1 0 1 1", vec![vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]] ; "repeated command")]
    #[test_case("M 5 5", vec![] ; "move only")]
    #[test_case("", vec![] ; "empty")]
    fn subpaths(d: &str, expected: Vec<Vec<(f64, f64)>>) {
        assert_eq!(points(d), expected);
    }

    /// Every curve is cubic, a line has its handles at a third of it.
    #[test]
    fn cubic_lines() {
        let subpaths = parse_path("M 0 0 L 30 0").expect("A path");
        assert_eq!(
            cubic_points(&subpaths[0][0]),
            [[0.0, 0.0], [10.0, 0.0], [20.0, 0.0], [30.0, 0.0]]
        );
    }

    /// Curves keep their handles, quadratic and smooth curves become the same cubic curves.
    #[test]
    fn cubic_curves() {
        let handles = |d: &str| {
            let points = cubic_points(parse_path(d).expect("A path")[0].last().expect("A curve"));
            (points[1], points[2])
        };
        assert_eq!(
            handles("M 0 0 C 0 10 20 10 20 0"),
            ([0.0, 10.0], [20.0, 10.0])
        );
        // The handle of the quadratic curve at two thirds from the start and from the end.
        let (start, end) = handles("M 0 0 Q 15 15 30 0");
        assert_eq!((start, end), ([10.0, 10.0], [20.0, 10.0]));
        // The first handle of a smooth curve is the reflection of the last handle before.
        assert_eq!(
            handles("M 0 0 C 0 10 20 10 20 0 S 40 -10 40 0"),
            ([20.0, -10.0], [40.0, -10.0])
        );
    }

    /// An arc becomes cubic curves on the circle of the arc.
    #[test]
    fn arc() {
        let subpaths = parse_path("M 0 0 A 5 5 0 0 1 10 0").expect("A path");
        let curves = &subpaths[0];
        assert!(!curves.is_empty());
        let end = Point::from(curves.last().expect("A curve").end);
        assert!(distance(end, [10.0, 0.0]) < 1e-9);
        // Points of the curves are on the circle around (5, 0) with radius 5.
        for curve in curves {
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let point = Point::from(curve.evaluate(TValue::Parametric(t)));
                let radius = distance(point, [5.0, 0.0]);
                assert!((radius - 5.0).abs() < 0.01, "{point:?} has radius {radius}");
            }
        }
    }

    #[test_case("M 0 0 L 10" ; "missing number")]
    #[test_case("M 0 0 X 10 10" ; "unknown command")]
    #[test_case("L 10 10" ; "no move at the start")]
    fn invalid(d: &str) {
        assert!(
            matches!(parse_path(d), Err(SvgError::InvalidPath(_))),
            "{d}"
        );
    }
}
