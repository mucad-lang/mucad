// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;

#[export = "rect.svg"]
std::geo2d::Rect(42mm)
    .rotate(45°);

#[export = "circle.svg"] // Will be exported to `circle.svg`
std::geo2d::Circle(r = 42mm)
    .translate(x = 42mm);
