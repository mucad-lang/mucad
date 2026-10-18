// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later
// file: logo.mu

pub mod mucad {
    use std::geo2d::*;
    use std::ops::*;

    sketch IconElement(radius: Length) {
        c = Sector(radius, start = 180°, end = 270°).translate(y = radius);
        r = Rect(width = radius, height = radius * 2, x = -radius, y = -radius);
        c | r
    }

    sketch Arc(radius: Length, thickness: Length, angle = 300°) {
        Sector(radius, start = -angle / 2, end = angle / 2)
            .rotate(180° - angle) - Circle(radius = radius - thickness)
    }

    pub sketch Icon(radius: Length) {
        prop gap = radius / 5;
        IconElement(radius)
            .translate(x = -radius - gap)
            .rotate([0..3] * 90°)
    }

    pub sketch Micro(radius: Length, thickness: Length) {
        u = Sector(radius, 180°).rotate(90°) - Circle(radius = radius - thickness);
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
        u | r_l | r_r
    }

    pub sketch C(radius: Length, thickness: Length) {
        Arc(radius, thickness, 260°)
    }

    pub sketch A(radius: Length, thickness: Length) {
        Arc(radius, thickness)
        | Rect(
            width = thickness,
            height = radius * 2,
            x = radius * 0.5,
            y = -radius,
        )
    }

    pub sketch D(radius: Length, thickness: Length) {
        Arc(radius, thickness)
        | Rect(
            width = thickness,
            height = radius * 3,
            x = radius * 0.5,
            y = -radius,
        )
    }

    pub sketch Logo(radius: Length, thickness: Length) {
        use std::math::*;
        init(size: Length) {
            radius = size * 50%;
            thickness = radius * 40%;
        }

        {
            Icon(radius);
            {
                Micro(radius, thickness);
                C(radius, thickness);
                A(radius, thickness);
                D(radius, thickness);
            }.align(X);
        }.align(X, 4mm).center()
    }
}

use std::ops::*;

pub const SIZE = 1cm;

mucad::Logo(size = SIZE).extrude(4mm);
