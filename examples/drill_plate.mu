// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

//! A template for a drill plate. The four holes are produced via list expansion.

/// A drill plate with four holes.
sketch DrillPlate(size = 100mm, hole_size = 10mm) {
    l = size / 2 - hole_size;
    holes = Circle(diameter = hole_size).translate(x = [-l, l], y = [-l, l]);
    Rect(size) - holes;
}

DrillPlate();
