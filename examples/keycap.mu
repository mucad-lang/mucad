// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::buffer;
use std::ops::contour;

/// The µ shape.
pub sketch Mu(radius: Length, thickness: Length) {
    use std::geo2d::Sector;
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

op smooth(r: Length) {
    @input.buffer(r).buffer(-r);
}

sketch MountProfile() {
    const SIZE = 4.1mm;
    const WIDTH = 1.17mm;
    const CORNER_RADIUS = 0.3mm;

    cross = Rect(width = WIDTH, height = SIZE).rotate([0, 90]°).smooth(CORNER_RADIUS);

    Circle(d = SIZE + WIDTH) - cross;

}

const KEYCAP_SIZE = 18mm;
const BASE_HEIGHT = 5mm;
const THICKNESS = 1mm;

sketch KeyCapBaseProfile() {
    MountProfile() |
    Rect(KEYCAP_SIZE - THICKNESS).smooth(-1mm).contour(THICKNESS);
}

{
    KeyCapBaseProfile().extrude(BASE_HEIGHT);


    cap_profile = Rect(KEYCAP_SIZE - 1mm).smooth(-1mm).buffer(0.5mm);

    cap_profile.extrude(THICKNESS);
    (cap_profile - Mu(4.0mm, 1mm).std::ops::center()).extrude(THICKNESS);
}.std::ops::align(Z).union();
