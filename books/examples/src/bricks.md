# Example: bricks

## Module: brick

[![test](.test/bricks_brick.svg)](.test/bricks_brick.log)

```mu,bricks_brick
// file: brick.µcad
// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;
use std::math::*;

pub const SPACING = 8mm;
pub const THICKNESS = 1.2mm;
pub const BASE_HEIGHT = 9.6mm;
pub const TOLERANCE = 0.2mm;

pub part Brick(rows = 2, columns = 4, base_height = BASE_HEIGHT) {
    width = columns * SPACING - TOLERANCE;
    height = rows * SPACING - TOLERANCE;

    base = {
        Frame(width, height, THICKNESS);
        r = rows - 1;
        c = columns - 1;
        n_rings = r * c;
        if n_rings > 0 {
            Ring(outer_diameter = 6.51mm, inner_diameter = 4.8mm)
                .multiply(n_rings)
                .distribute_grid(
                    width = width - width / columns,
                    height = height - height / rows,
                    rows = r,
                    columns = c,
                );
        }
    }
    .extrude(base_height - THICKNESS);

    cap = Rect(width, height).extrude(THICKNESS);

    knobs = Circle(d = 4.8mm)
        .multiply(rows * columns)
        .distribute_grid(width, height, rows, columns)
        .extrude(1.7mm);

    {
        base; cap; knobs;
    }.align(Z).union();
}

n = 4;

Brick(
    rows = [1..n],
    columns = [1..n],
    base_height = BASE_HEIGHT * [1 / 3, 100%, 200%, 300%],
)
    .distribute_grid(cell_size = n * 10mm, rows = 2 * n, columns = 2 * n);

```

**2D Output**
    : ![None](.test/bricks_brick-out.svg)

**3D Output**
    : ![None](.test/bricks_brick-out.stl)

## Module: tutorial

[![test](.test/bricks_tutorial.svg)](.test/bricks_tutorial.log)

```mu,bricks_tutorial
// file: tutorial.µcad
// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;

pub const SPACING = 8mm;

op grid(columns: Integer, rows: Integer) {
    @input
        .translate(x = [1..columns] * SPACING, y = [1..rows] * SPACING)
        .center()
}

sketch Base(columns: Integer, rows: Integer, width: Length, height: Length) {
    thickness = 1.2mm;
    frame = Frame(width, height, thickness);
    struts = Ring(outer_diameter = 6.51mm, inner_diameter = 4.8mm)
        .grid(columns = columns - 1, rows = rows - 1);
    frame | struts;
}

use Rect as Cap;

sketch Knobs(columns: Integer, rows: Integer) {
    Circle(d = 4.8mm)
        .grid(columns, rows);
}

pub part LegoBrick(rows = 2, columns = 4, base_height = 9.6mm) {
    width = columns * SPACING - 0.2mm;
    height = rows * SPACING - 0.2mm;
    cap_thickness = 1.0mm;

    base = Base(rows, columns, width, height)
        .extrude(base_height - cap_thickness);

    cap = Cap(width, height)
        .extrude(cap_thickness)
        .translate(z = base_height - cap_thickness);

    knobs = Knobs(rows, columns)
        .extrude(1.7mm)
        .translate(z = base_height);

    base | cap | knobs;
}

// render a brick with default values
LegoBrick();

```

**2D Output**
    : ![None](.test/bricks_tutorial-out.svg)

**3D Output**
    : ![None](.test/bricks_tutorial-out.stl)

## Module: use_bricks

[![test](.test/bricks_use_bricks.svg)](.test/bricks_use_bricks.log)

```mu,bricks_use_bricks
// file: use_bricks.µcad
// Copyright © 2025 The µcad authors <info@microcad.xyz>
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
```

**2D Output**
    : ![None](.test/bricks_use_bricks-out.svg)

**3D Output**
    : ![None](.test/bricks_use_bricks-out.stl)
