// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::geo3d::*;
use std::ops::*;
use std::math::*;

const THICKNESS = 2mm;

part Hole(diameter: Length) {
    r = diameter * 50%;
    Cylinder(
        radius_bottom = r,
        radius_top = r + THICKNESS * 50%,
        height = THICKNESS,
    );
}

r = RoundedRect(60mm, radius = 10mm).extrude(THICKNESS);

holes = Hole([1..9] * 0.25mm)
    .distribute_grid(size = 60mm, rows = 3, columns = 3);

r - holes;
