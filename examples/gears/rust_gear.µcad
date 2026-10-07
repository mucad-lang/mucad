// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;
use std::geo2d::Circle;

op polar_array(n: Integer, r: Length) {
    @input.translate(x = r)
        .rotate([1..n] / n * 360°)
}

sketch RustGear(r = 50mm) {
    r_i = r * 80%;

    rim = Circle(r) - Circle(r = r_i);

    tooth = Circle(r * 2%).polar_array(3, r * 10%).hull();
    teeth = tooth.polar_array(32, r);

    hub = Circle(r * 4%).polar_array(4, r * 10%).hull();
    hubs = hub.polar_array(5, r_i);

    holes = Circle(r * 6%).polar_array(5, r_i);

    rim | teeth | hubs - holes;
}

RustGear().extrude(10mm);

mod rust_r;
rust_r::RustR().extrude(5mm);
