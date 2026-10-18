// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later
// file: dome.mu

pub part Dome(size: Length, strut_width = 1mm) {
    Rect(strut_width)
        .translate(x = size)
        .revolve(180°)
        .rotate(x = -[0..6] * 180° / 6)
        .rotate(z = [0..3] * 180° / 3);
}

Dome(10mm, strut_width = 0.25mm);
