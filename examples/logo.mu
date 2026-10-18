// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later
// file: logo.mu

/// Some module with shapes.
pub mod shapes {
    use std::geo2d::*;
    use std::ops::*;

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
        (u | r_l | r_r).buffer(6.0mm).scale(100%);
    }

    /// "C" shape.
    pub sketch C(radius: Length, thickness: Length) {
        Arc(radius, thickness, 260°).buffer(6.0mm)
    }

    /// "A" shape.
    pub sketch A(radius: Length, thickness: Length) {
        Arc(radius, thickness)
        | Rect(
            width = thickness,
            height = radius * 2,
            x = radius * 0.75,
            y = -radius,
        );
    }

    /// "D" shape.
    pub sketch D(radius: Length, thickness: Length) {
        Arc(radius, thickness)
        | Rect(
            width = thickness,
            height = radius * 3,
            x = radius * 0.75,
            y = -radius,
        );
    }

    /// Private helper: An Arc struct.
    sketch Arc(radius: Length, thickness: Length, angle = 300°) {
        Sector(radius, start = -angle / 2, end = angle / 2).rotate(180°)
            - Circle(radius = radius - thickness);
    }
}

sketch WithMuGear(radius: Length, model: Model2D) {
    (std::geo2d::SinusoidalGearProfile(10mm, 13) - Circle(radius)) | model;
}

// A constant symbol for the overall size.
const SIZE = 10cm;

// Pick any shape you like.
shape = shapes::Mu(SIZE * 50%, SIZE * 10%);
// shape = shapes::C(SIZE * 50%, SIZE * 10%);
// shape = shapes::A(SIZE * 50%, SIZE * 10%);
// shape = shapes::D(SIZE * 50%, SIZE * 10%);
// shape = Rect(SIZE);

// Let's turn it 3D.
WithMuGear(SIZE, shape).extrude(10mm);
