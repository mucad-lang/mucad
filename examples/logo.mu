// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A µcad gear logo with a little shape module. The interesting stuff happens at the bottom.

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
    (u | r_l | r_r).buffer(8.0mm).scale(100%);
}

// The official µcad gear (private sketch)
sketch MucadLogo(size: Length) {
    gear = (std::geo2d::SinusoidalGearProfile(10mm, 13) - Circle(size)).rotate(7°);
    gear | Mu(SIZE * 50%, SIZE * 10%);
}

// A constant symbol for the overall size.
const SIZE = 10cm;

MucadLogo(SIZE).extrude(10mm);
