// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;

/// A drill plate with four holes.
#[color = "red"]
sketch DrillPlate(size = 100mm, hole_size = 10mm) {
    l = size / 2 - hole_size;
    Rect(size) - Circle(diameter = hole_size).translate(x = [-l, l], y = [-l, l])
}

DrillPlate();
