// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: CC_BY
//
// The Rust logo is a registered trademark of the Rust Foundation.

sketch RustGear(r = 50mm) {
    use std::ops::polar_array;

    r_i = r * 80%;

    rim = Circle(r) - Circle(r = r_i);

    tooth = Circle(r * 2%).polar_array(3, r * 10%).hull();
    teeth = tooth.polar_array(32, r);

    hub = Circle(r * 4%).polar_array(4, r * 10%).hull();
    hubs = hub.polar_array(5, r_i);

    holes = Circle(r * 6%).polar_array(5, r_i);

    rim | teeth | hubs - holes;
}

r = std::geo2d::SvgPath(
    "M -9,-15 H 4 C 12,-15 12,-7 4,-7 H -9 Z M -40,22 H 0 V 11 H -9 V 3 H 1 C 12,3 6,22 15,22 H 40 V 3 H 34 V 5 C 34,13 25,12 24,7 C 23,2 19,-2 18,-2 C 33,-10 24,-26 12,-26 H -35 V -15 H -25 V 11 H -40 Z",
).rotate(270°);

RustGear().extrude(10mm) | r.extrude(5mm);
