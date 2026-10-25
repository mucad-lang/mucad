// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;

pub const SPACING = 8mm;
pub const THICKNESS = 1.2mm;
pub const BASE_HEIGHT = 9.6mm;
pub const TOLERANCE = 0.2mm;

pub part Brick() {
    width = 2 * SPACING - TOLERANCE;
    height = 4 * SPACING - TOLERANCE;

    base = {
        Frame(width, height, THICKNESS);
        Ring(outer_diameter = 6.51mm, inner_diameter = 4.8mm)
            .multiply(3)
            .distribute_grid(
                width = 0cm,
                height = height - height / 4,
                rows = 3,
                columns = 1,
            );
    }
    .extrude(BASE_HEIGHT - THICKNESS);

    cap = Rect(width, height).extrude(THICKNESS);

    knobs = Circle(d = 4.8mm)
        .multiply(8)
        .distribute_grid(width, height, 4, 2)
        .extrude(1.7mm);

    { base; cap; knobs; }.align(Z).union();
}

#[resolution = 200%]
Brick();
