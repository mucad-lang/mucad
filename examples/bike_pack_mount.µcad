// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;
use std::math::*;

sketch Base(width = 130.0mm, height = 23.5mm, radius = 3mm) {
    RoundedRect(width, height, radius);
}

part RackRails(rack_rail_distance = 100mm, rack_rail_diameter = 10.5mm) {
    {
        rd = (rack_rail_distance + rack_rail_diameter) / 2;
        Circle(d = rack_rail_diameter, c = (x = [-0.5, 0.5]mm, y = 0mm))
            .hull()
            .translate(x = [-rd, rd]);
    }
    .extrude(40mm)
    .orient(Y)
    .translate(y = -15mm);
}

part Mount(
    height: Length,
    screw_distance = 94mm,
    screw_hole_diameter = 6mm,
    rack_rail_distance = 100mm,
    rack_rail_diameter = 10.5mm,
) {
    d = screw_distance / 2;
    screws = Circle(d = screw_hole_diameter, c = (x = [-d, d], y = 0.0mm));

    Base().extrude(height) - screws.extrude(height + 10mm) - RackRails();
}

pub part Lower() {
    Mount(height = 15mm);
}

pub part Upper() {
    Mount(height = 55mm);
}

#[export = "lower.stl"]
Lower().translate(y = 50mm);

#[export = "upper.stl"]
Upper().translate(y = -50mm);
