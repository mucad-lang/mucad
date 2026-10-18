// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

#[export = "rect.svg"]
Rect(42mm)
    .rotate(45°);

#[export = "circle.svg"] // Will be exported to `circle.svg`
Circle(r = 42mm)
    .translate(x = 42mm);
