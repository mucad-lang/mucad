// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

//! A none-geodesic dome constructed via list expansions.

pub part Dome(size: Length, strut_width = 1mm) {
    Rect(strut_width)
        .translate(x = size)
        .revolve(180°)
        .rotate(x = [0..6] * 180° / 6)
        .rotate(z = [0..3] * 180° / 3)
        .union();
}

Dome(50mm, strut_width = 2.5mm);
