// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;
use std::math::*;

pub const SPACING = 8mm;
pub const THICKNESS = 1.2mm;
pub const BASE_HEIGHT = 9.6mm;
pub const TOLERANCE = 0.2mm;

pub part Brick(rows = 2, columns = 4, base_height = BASE_HEIGHT) {
    width = columns * SPACING - TOLERANCE;
    height = rows * SPACING - TOLERANCE;

    base = {
        Frame(width, height, THICKNESS);
        r = rows - 1;
        c = columns - 1;
        n_rings = r * c;
        if n_rings > 0 {
            Ring(outer_diameter = 6.51mm, inner_diameter = 4.8mm)
                .multiply(n_rings)
                .distribute_grid(
                    width = width - width / columns,
                    height = height - height / rows,
                    rows = r, 
                    columns = c
                );
        }
    }
    .extrude(base_height - THICKNESS);

    cap = Rect(width, height).extrude(THICKNESS);

    knobs = Circle(d = 4.8mm)
        .multiply(rows * columns)
        .distribute_grid(width, height, rows, columns)
        .extrude(1.7mm);

    { base; cap; knobs; }.align(Z).union();
}

n = 4;

Brick(
    rows = [1..n], 
    columns = [1..n], 
    base_height = BASE_HEIGHT * [1 / 3, 100%, 200%, 300%]
)
.distribute_grid(cell_size = n * 10mm, 
    rows = 2*n, 
    columns = 2*n,
);
