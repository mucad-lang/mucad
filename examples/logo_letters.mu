// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The µcad logo.

/// Some public module with shapes used by the logos.
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
        (u | r_l | r_r).buffer(thickness * 75%);
    }

    /// "C" shape.
    pub sketch C(radius: Length, thickness: Length) {
        Arc(radius, thickness, 260°).buffer(thickness * 75%)
    }

    /// "A" shape.
    pub sketch A(radius: Length, thickness: Length) {
        (Arc(radius, thickness)
        | Rect(
            width = thickness,
            height = radius * 2,
            x = radius * 0.75,
            y = -radius,
        )).buffer(thickness * 75%);
    }

    /// "D" shape.
    pub sketch D(radius: Length, thickness: Length) {
        (Arc(radius, thickness)
        | Rect(
            width = thickness,
            height = radius * 3,
            x = radius * 0.75,
            y = -radius,
        )).buffer(thickness * 75%);
    }

    /// An Arc struct.
    pub sketch Arc(radius: Length, thickness: Length, angle = 310°) {
        Sector(radius, start = -angle / 2, end = angle / 2).rotate(180°)
        - Circle(radius = radius - thickness);
    }

    // The official µcad gear (private sketch)
    pub sketch MucadGear(size: Length) {
        (std::geo2d::SinusoidalGearProfile(size * 5%, 13) - Circle(d = size)).rotate(7°);
    }
}

// The official µcad logo (public sketch for re-use)
pub sketch MucadLogo(size: Length) {
    shapes::MucadGear(size) | shapes::Mu(size * 25%, size * 5%);
}

// The official µcad logo (public sketch for re-use)
pub sketch MucadLogoBig(size: Length) {
    use std::ops::*;
    cutout = std::geo2d::Sector(size * 25%, start = -260° / 2, end = 260° / 2).rotate(180°).buffer(size * 5% + 1mm);

    letters = {
        shapes::Mu(size * 25%, size * 5%);
        shapes::C(size * 25%, size * 5%);
        shapes::A(size * 25%, size * 5%);
        shapes::D(size * 25%, size * 5%);
    }.std::ops::align(std::math::X, spacing = 2mm).translate(x = -size * 28.5%);

    (shapes::MucadGear(size) - cutout.translate(x = size * 60%)) | letters; }

// A constant symbol for the overall size.
const SIZE = 10cm;

MucadLogoBig(SIZE);
