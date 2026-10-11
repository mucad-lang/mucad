# Example: letter_cube

[![test](.test/letter_cube.svg)](.test/letter_cube.log)

```mu,letter_cube
// Copyright © 2025 The µcad authors <info@microcad.xyz>
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
        .orient(axis);
}

part LetterCube(size: Length) {
    s = size / sqrt(2.1);
    body = Cube(size) & Sphere(r = s);
    p = Letter("P", size, [-Z, Z]).rotate(z = -90°);
    t = Letter("T", size, [-X, X]).rotate(x = 90°);
    f = Letter("F", size, [-Y, Y]);
    body - (p | t | f);
}

LetterCube(20mm).translate(z = 10mm);

```

**2D Output**
    : ![None](.test/letter_cube-out.svg)

**3D Output**
    : ![None](.test/letter_cube-out.stl)
