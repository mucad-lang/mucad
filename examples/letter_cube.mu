// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::math::*;
use std::ops::*;
use std::geo3d::*;
use std::geo2d::*;

part Letter(letter: String, size: Length, axis: Vec3) {
    Text(letter, height = size)
        .center()
        .contour(thickness = 0.25mm, distance = 0.5mm)
        .extrude(1mm)
        .translate(z = size / 2 - 0.5mm)
        .orient(axis)
}

part LetterCube(size: Length) {
    s = size / sqrt(2.1);
    body = Cube(size) & Sphere(r = s);
    p = Letter("P", size, [-Z, Z]).rotate(z = -90°);
    t = Letter("T", size, [-X, X]).rotate(x = 90°);
    f = Letter("F", size, [-Y, Y]);
    body - (p | t | f)
}

LetterCube(20mm).translate(z = 10mm);
