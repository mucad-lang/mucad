// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

use std::geo2d::*;
use std::ops::*;

sketch Hello(height = 42mm) {
    RoundedRect(width = 4 * height, height, radius = 10mm) - Text(height, "Hello µcad!").center();
}

// create a 3D text
Hello().extrude(23mm);
