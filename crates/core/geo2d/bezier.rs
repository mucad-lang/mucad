// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bezier implementation

pub use bezier_rs::{Bezier, Identifier, TValue};
use geo::{Coord, Polygon};

/// An empty id type for use in tests
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct EmptyId;

impl Identifier for EmptyId {
    fn new() -> Self {
        Self
    }
}

/// Subpath with empty id
pub type Subpath = bezier_rs::Subpath<EmptyId>;

/// Create a polygon from a Bezier path
pub fn subpath_to_geo_polygon(subpath: &Subpath, steps: usize) -> Polygon {
    let mut coordinates = Vec::new();

    // 1. .iter() still yields an iterator over Bezier segments (SubpathIter),
    // which automatically accounts for the closing segment if `subpath.closed` is true.
    for bezier_segment in subpath.iter() {
        // 2. Sample along the segment parametric timeline (0.0 to 1.0)
        for i in 0..steps {
            let t = i as f64 / steps as f64;
            let point = bezier_segment.evaluate(TValue::Parametric(t));
            coordinates.push(Coord {
                x: point.x,
                y: point.y,
            });
        }
    }

    // 3. Close the loop if required by geo::LineString
    if subpath.closed && !coordinates.is_empty() {
        coordinates.push(coordinates[0]);
    } else if let Some(last_bezier) = subpath.iter().last() {
        // If open, ensure we explicitly grab the absolute terminal point
        let end_point = last_bezier.evaluate(TValue::Parametric(1.0));
        coordinates.push(Coord {
            x: end_point.x,
            y: end_point.y,
        });
    }

    Polygon::new(coordinates.into(), vec![])
}
