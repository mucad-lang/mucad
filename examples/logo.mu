// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

//! The µcad gear logo as sketch.

use std::geo2d::Sector;
use std::ops::buffer;

/// The µ shape.
pub sketch Mu(radius: Length, thickness: Length) {
    u = Sector(radius, 180°).rotate(-90°) - Circle(r = radius - thickness);
    r_l = Rect(
        width = thickness,
        height = radius * 3,
        x = -radius,
        y = -2 * radius,
    );

    r_r = Rect(
        width = thickness,
        height = radius * 2,
        x = radius - thickness,
        y = -radius,
    );
    (u | r_l | r_r).buffer(thickness * 75%);
}

// The official µcad gear (private sketch)
sketch MucadGear(size: Length) {
    (std::geo2d::SinusoidalGearProfile(size * 5%, 13) - Circle(d = size)).rotate(7°);
}

// The official µcad logo (public sketch for re-use)
pub sketch MucadLogo(size: Length) {
    MucadGear(size) | Mu(size * 25%, size * 5%);
}

// A constant symbol for the overall size.
const SIZE = 10cm;

MucadLogo(SIZE);

// You can extrude it and produce a 3D version
// MucadLogo(SIZE).extrude(10mm);
