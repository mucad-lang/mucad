// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! 3D plane geometry.

use crate::Vec3;

/// A 3D plane with normal and point.
pub struct Plane {
    /// Plane origin point.
    pub p: Vec3,
    /// Plane normal.
    pub n: Vec3,
}
