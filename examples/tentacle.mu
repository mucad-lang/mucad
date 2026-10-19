// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Day of the Tentacle

use std::math::*;
use std::ops::align;

sketch RoundedRect(r: Length, w: Length) {
    Circle(r).translate(x = [w,-w], y = [w, -w]);
}

part Slice(f: Scalar) {
   RoundedRect(r = (1.0 - f) * 5mm, w = f * 5mm).extrude(0.5mm).rotate(f * 720°);
}

n = 100;
Slice([1..n] / n).align(Z);
