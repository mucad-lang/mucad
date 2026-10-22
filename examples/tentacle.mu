// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Day of the Tentacle

use std::math::*;
use std::ops::align;

part Slice(f: Scalar) {
   Circle(r = (1.0 - f) * 5mm).translate(x = f * 5mm).extrude(0.5mm).rotate(f * 720°);
}

n = 100;
Slice([1..n] / n).align(Z);
