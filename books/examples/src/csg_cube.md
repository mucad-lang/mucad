# Example: csg_cube

[![test](.test/csg_cube.svg)](.test/csg_cube.log)

```mu,csg_cube
// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::math::*;
use std::geo3d::*;

part CsgCube(size: Length) {
    s = size / sqrt(2.1);
    body = Sphere(radius = s) & Cube(size);
    holes = Cylinder(size, diameter = s).orient([X, Y, Z]);
    body - holes;
}

CsgCube(50mm);

```

**2D Output**
    : ![None](.test/csg_cube-out.svg)

**3D Output**
    : ![None](.test/csg_cube-out.stl)
