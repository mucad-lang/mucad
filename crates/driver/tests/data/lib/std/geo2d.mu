// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

/// A circle with a radius.
pub sketch Circle(radius: Length) {
    __mu::geo2d::Circle(radius = radius);
}

/// The default radius of circles.
pub const RADIUS = 1mm;
