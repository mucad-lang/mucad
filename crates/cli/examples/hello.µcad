// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;

sketch Hello(height = 42mm) {
    RoundedRect(width = 4 * height, height, radius = 10mm) - Text(height, "Hello µcad!").center();
}

// create a 3D text
Hello().extrude(23mm);
