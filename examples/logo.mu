// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later
// file: logo.mu

pub mod mucad {
    use std::geo2d::*;
    use std::ops::*;

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
        (u | r_l | r_r).buffer(3.0mm)
    }

    pub sketch Logo(radius: Length, thickness: Length) {
        use std::math::*;
        init(size: Length) {
            radius = size * 50%;
            thickness = size * 20%;
        }

        SinusoidalGearProfile(10mm, 13) - Circle(radius * 200.0%);
        Mu(radius, thickness);
    }
}

pub const SIZE = 10cm;

mucad::Logo(size = SIZE).extrude(4mm);
(Circle(r = SIZE) - Circle(r = SIZE * 80%)).extrude(2mm);
