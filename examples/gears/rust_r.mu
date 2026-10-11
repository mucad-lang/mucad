// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::Circle;
use std::ops::*;

pub sketch RustR() {
    r = __builtin::lab::BezierPolygon(
        "-40 -22 -40 -22  0 -22 0 -22  0 -22  0 -11 0 -11  0 -11 -9 -11 -9 -11 -9 -11 -9  -3 -9  -3 -9  -3 1  -3   1  -3  12  -3  6 -22  15 -22  15 -22 40 -22  40 -22  40 -22 40  -3  40  -3  40  -3 34  -3  34  -3  34  -3 34  -5  34  -5  34 -13 25 -12  24  -7  23  -2 19   2  18   2  33  10 24  26  12  26  12  26 -35  26 -35  26 -35  26 -35  15 -35  15 -35  15 -25  15 -25  15 -25  15 -25 -11 -25 -11 -25 -11 -40 -11 -40 -11",
    );
    r_hole = __builtin::lab::BezierPolygon("-9 15 -9 15 4 15  4 15 12 15 12  7  4  7  4  7 -9  7 -9  7");

    (r - r_hole).buffer(1mm).rotate(-90°)
}
