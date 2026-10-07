// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;
use std::geo2d::*;
use std::math::*;

part Gear(teeth: Integer)  {
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
    .translate(gear1.outer_radius + gear2.pitch_radius, [1..3] * 360° / 3);


