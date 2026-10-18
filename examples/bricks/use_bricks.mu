// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod brick;

use brick::*;

// 2x2 double height
double_2x2 = Brick(rows = 2, columns = 2, base_height = 9.6mm * 2);

// 4x2 single height
single_4x2 = Brick(rows = 4, columns = 2);

// 3x2 one-third height
third_3x2 = Brick(rows = 3, columns = 2, base_height = 3.2mm);

// generate geometry placing all elements side by side
use std::ops::translate;

single_4x2;
double_2x2.translate(y = -40mm);
third_3x2.translate(y = 40mm);
