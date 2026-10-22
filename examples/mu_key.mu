// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later


use std::ops::*;
use std::geo2d::*;

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
    (u | r_l | r_r).buffer(thickness * 25%).scale(100%);
}

// The official µcad gear (private sketch)
sketch MucadLogo(size: Length) {
    gear = (std::geo2d::SinusoidalGearProfile(size * 10%, 13) - Circle(size)).rotate(7°);
    gear | Mu(size * 50%, size * 10%);
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
    Rect(KEYCAP_SIZE).smooth(-1.5mm).extrude(THICKNESS).translate(z = BASE_HEIGHT - THICKNESS);

    #[resolution = 300%]
    (Rect(h = 1mm, w = KEYCAP_SIZE / 2 - 1mm, x = 0.0mm, y = 0.0mm) - Circle(r = 1mm).translate(x = KEYCAP_SIZE / 2.0 - 1mm, y = 1mm)).revolve(360°);

    Circle(d = KEYCAP_SIZE - THICKNESS * 4).extrude(1mm) - MucadLogo(4mm).extrude(1mm).translate(z = 0.7mm);
}.align(std::math::Z);

//;
//
