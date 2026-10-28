// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

use std::ops::*;

const SPACING = 8mm;
const THICKNESS = 1.2mm;
const BASE_HEIGHT = 9.6mm;
const TOLERANCE = 0.2mm;

op array(rows: Integer, columns: Integer) {
    @input.multiply(rows * columns).distribute_grid(rows, columns, SPACING * columns, SPACING * rows);
}

part Brick(columns: Integer, rows: Integer, base_height: Length) {
    width = columns * SPACING - TOLERANCE;
    height = rows * SPACING - TOLERANCE;

    rect = Rect(width, height);
    frame = rect - rect.buffer(-THICKNESS);

    ring = Circle(d = 6.51mm) - Circle(d = 4.8mm);
    rings = ring.array(rows - 1, columns - 1);

    base = (frame | rings).extrude(base_height - THICKNESS);

    cap = Rect(width, height).extrude(THICKNESS);

    knobs = Circle(d = 4.8mm).array(columns, rows)
        .extrude(1.7mm);

    {
        base;
        cap;
        knobs;
    }.align(Z);
}

Brick([1..4], [1..4], BASE_HEIGHT * [1..4]).distribute_grid(27cm, 27cm, 8, 8);
