// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

//! For the web-viewer: Go with the cursor above any literal (e.g. `45mm`)
//! and use up/down arrows + Alt key to fiddle change the parameter values.

use std::ops::*;

/// A part definition for a parametric footpad mount.
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

    // Create 4 holes
    hole_pos = (x = [-1, 1] * s.x, y = [-1, 1] * s.y);
    radius = corner_radius;

    // Create the base plate with 4 rounded corners.
    base_lower = Circle(radius, c = hole_pos)
        .hull()
        .extrude(12mm);

    base_lower_height = base.z - slot.x;

    center_pole = {
        Circle(d = center_hole.d_outer)
            .extrude(height);

        // Cut off on top.
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

    base_lower | center_pole - holes;
}

FootPad();
