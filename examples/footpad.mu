// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

use std::ops::*;

part FootPad(
    height = 45mm,
    base = (x = 155mm, y = 101mm, z = 18mm),
    slot = (x = 8mm, y = 13mm),
    center_hole = (d_inner = 42mm, d_outer = 55mm),
    corner_hole = (d_lower = 14mm, d_upper = 22mm),
    corner_radius = 15mm,
) {
    s = (
        x = base.x / 2 - corner_radius,
        y = base.y / 2 - corner_radius,
    );
    hole_pos = (x = [-1, 1] * s.x, y = [-1, 1] * s.y);
    radius = corner_radius;

    base_lower = Circle(radius, c = hole_pos)
        .hull()
        .extrude(12mm);

    base_upper_height = slot.x;
    base_lower_height = base.z - base_upper_height;

    center = {
        (Circle(d = center_hole.d_outer) - Circle(d = center_hole.d_inner))
            .extrude(height);

        Rect(height = slot.x, width = center_hole.d_outer)
            .extrude(slot.y)
            .translate(z = height - slot.y);

    }.subtract();

    holes = {
        Circle(d = corner_hole.d_lower, c = hole_pos)
            .extrude(height = base_lower_height);
        Circle(d = corner_hole.d_upper, c = hole_pos)
            .extrude(height = base.z * 2)
            .translate(z = base_lower_height);
        Circle(d = center_hole.d_inner)
            .extrude(height);
    }.union();

    base_lower | center - holes;
}

FootPad();
