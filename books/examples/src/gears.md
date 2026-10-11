# Example: gears

## Module: gear_2d

[![test](.test/gears_gear_2d.svg)](.test/gears_gear_2d.log)

```mu,gears_gear_2d
// file: gear_2d.µcad
// Copyright © 2025 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;
use std::geo2d::*;

InvoluteGearProfile(1.0mm, 100);

```

**2D Output**
    : ![None](.test/gears_gear_2d-out.svg)

**3D Output**
    : ![None](.test/gears_gear_2d-out.stl)

## Module: gears_3d

[![test](.test/gears_gears_3d.svg)](.test/gears_gears_3d.log)

```mu,gears_gears_3d
// file: gears_3d.µcad
// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;
use std::geo2d::*;
use std::math::*;

const TEETH = 20;

angle = 30° * TEETH;

part Gear(teeth: Integer) {
    gear_profile = InvoluteGearProfile(module = 1.0mm, teeth);

    prop pitch_radius = gear_profile.pitch_radius;
    prop outer_radius = gear_profile.outer_radius;

    n_holes = 5;

    bearing_hole = Circle(radius = gear_profile.pitch_radius / 8);

    hole_radius = pitch_radius / n_holes;
    holes = Circle(radius = pitch_radius / n_holes)
        .translate(x = outer_radius - hole_radius * 2)
        .rotate([1..n_holes] * 360° / n_holes);

    inner = Circle(radius = pitch_radius - 3mm);

    bearing = (Circle(radius = pitch_radius / 4) - bearing_hole).extrude(8mm)
    - Circle(radius = gear_profile.module)
        .extrude(pitch_radius / 4)
        .translate(z = -pitch_radius / 4)
        .rotate(x = 90°)
        .translate(z = 5mm);

    bearing
    | (inner - holes - Circle(radius = pitch_radius / 4)).extrude(2mm)
    | (gear_profile - inner).extrude(4mm);
}

gear1 = Gear(80);
gear2 = Gear(40);
gear3 = Gear(20);

#[color = std::color::BLUE]
#[export = "gear1.stl"]
gear1.rotate(angle / gear1.teeth);

#[color = std::color::RED]
#[export = "gear2.stl"]
gear2
    .rotate(-angle / gear2.teeth + 360° / gear2.teeth / 2.0)
    .translate(gear2.outer_radius + gear1.pitch_radius, 135°);

#[color = std::color::GREEN]
#[export = "gear3.stl"]
gear3
    .rotate(-angle / gear3.teeth)
    .translate(gear1.outer_radius + gear3.pitch_radius, 45°);


```

**2D Output**
    : ![None](.test/gears_gears_3d-out.svg)

**3D Output**
    : ![None](.test/gears_gears_3d-out.stl)

## Module: herringbone_gears

[![test](.test/gears_herringbone_gears.svg)](.test/gears_herringbone_gears.log)

```mu,gears_herringbone_gears
// file: herringbone_gears.µcad
// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;
use std::geo2d::*;
use std::math::*;

part Gear(teeth: Integer) {
    gear_profile = InvoluteGearProfile(1.5mm, teeth);
    prop outer_radius = gear_profile.outer_radius;
    prop pitch_radius = gear_profile.pitch_radius;
    prop tooth_angle = gear_profile.tooth_angle;
    prop tooth_distance = gear_profile.tooth_distance;
    prop module = gear_profile.module;

    disc = Circle(radius = pitch_radius - module * 400%);
    web = {
        sector = Sector(disc.radius, 360° / 5)
            .buffer(-disc.radius * 15%)
            .buffer(disc.radius * 12%);
        holes = (sector - Circle(radius = module * 1000%))
            .rotate([0..4] * 360° / 5);
        hub = InvoluteGearProfile(2.5mm, 7);
        disc - holes - hub;
    }.extrude(tooth_distance * 300%).center();

    rim = (gear_profile - disc)
        .extrude(tooth_distance * 200%, tooth_angle)
        .mirror(-Z);

    web | rim;
}

gear1 = Gear(90);
gear2 = Gear(60);

gear1.reflect(X);
gear2
    .rotate(gear2.tooth_angle * 50%)
    .translate(
        gear1.outer_radius + gear2.pitch_radius,
        [1..3] * 360° / 3,
    );


```

**2D Output**
    : ![None](.test/gears_herringbone_gears-out.svg)

**3D Output**
    : ![None](.test/gears_herringbone_gears-out.stl)
