# Example: ngons

[![test](.test/ngons.svg)](.test/ngons.log)

```mu,ngons
// Copyright © 2025 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;

(Hexagon(20mm) - Ngon(5, 10mm)).extrude(20mm);

```

**2D Output**
    : ![None](.test/ngons-out.svg)

**3D Output**
    : ![None](.test/ngons-out.stl)
