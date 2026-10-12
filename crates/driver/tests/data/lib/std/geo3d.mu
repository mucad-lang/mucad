// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// A cylinder, which refers to `std` itself like the real standard library.
pub part Cylinder(radius: Length, height: Length) {
    std::geo2d::Circle(radius)
        .__mu::ops::extrude(height, scale_x = 1.0, scale_y = 1.0, twist = 0°);
}
