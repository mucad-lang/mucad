// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

use std::math::*;
use std::geo3d::*;

part CsgCube(size: Length) {
    s = size / sqrt(2.1);
    body = Sphere(radius = s) & Cube(size);
    holes = Cylinder(size, diameter = s).orient([X, Y, Z]);
    body - holes;
}

CsgCube(50mm);
