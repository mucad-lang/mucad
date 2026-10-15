// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

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
    .rotate(-angle / gear2.teeth + 360°/gear2.teeth/2.0)
    .translate(gear2.outer_radius + gear1.pitch_radius, 135°);

#[color = std::color::GREEN]
#[export = "gear3.stl"]
gear3
    .rotate(-angle / gear3.teeth)
    .translate(gear1.outer_radius + gear3.pitch_radius, 45°);

