# Example: torus

[![test](.test/torus.svg)](.test/torus.log)

```mu,torus
// Copyright © 2025 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

std::geo3d::Torus([1..5] * 10mm, minor_radius = 3mm);

```

**2D Output**
    : ![None](.test/torus-out.svg)

**3D Output**
    : ![None](.test/torus-out.stl)
