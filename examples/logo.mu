// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

//! The µcad gear logo as sketch.

const SIZE = 10cm;

sketch Mu(size: Length) {
    r = size * 25%;
    w = size * 5%;

    use std::geo2d::Sector;
    u = Sector(r, 180°).rotate(-90°) - Circle(r - w);

    r_l = Rect(w, h = r * 3, x = -r, y = -2 * r);
    r_r = Rect(w, h = r * 2, x = r - w, y = -r);

    use std::ops::buffer;
    (r_l | r_r | u).buffer(w * 75%);
}

sketch MuGear(size: Length) {
    gear = std::geo2d::SinusoidalGearProfile(size * 5%, 13).rotate(7°);
    gear - Circle(d = size);
}

pub sketch MucadLogo(size: Length) {
    MuGear(size) | Mu(size);
}

#[resolution = 50%]
MucadLogo(SIZE).extrude(5mm);
