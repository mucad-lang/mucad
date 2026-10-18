// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Builtin align for 2D geometries.

use crate::*;

/// Trait to align a 2D geometry collection with spacing along an axis.
pub trait Align2D<T = Self> {
    /// Align geometry.
    fn align_2d(&self, direction: Vec2, spacing: Length) -> T;
}

impl Align2D for Geometries2D {
    fn align_2d(&self, direction: Vec2, spacing: Length) -> Self {
        Geometries2D::from_iter(self.iter().scan(0.0_f64, |pos, geo2d| {
            use cgmath::InnerSpace;

            let bounds = geo2d.calc_bounds_2d();
            let dist = *bounds.distance_center_to_boundary(direction);
            let dir = if direction.magnitude() > f64::EPSILON {
                direction.normalize()
            } else {
                Vec2::new(0.0, 0.0)
            };
            let d = (*pos + dist) * dir
                - Vec2::new(dir.x * bounds.center().x, dir.y * bounds.center().y);
            *pos += 2.0 * dist + *spacing;

            Some(geo2d.transformed_2d(&Mat3::from_translation(d)))
        }))
    }
}
