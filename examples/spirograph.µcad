// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;

mod logo;

use logo::microcad::Logo;

const TEETH_SIZE = 0.5mm;
const THICKNESS = 2mm;
const BORDER_WIDTH = 7mm;

sketch GearProfile(teeth: Integer) {
    prop module = TEETH_SIZE;
    prop radius = teeth * module;
    SinusoidalGearProfile(module, teeth, stretch = 0.6, roundness = 0.6);
}

sketch RingProfile(
    outer_teeth: Integer,
    inner_teeth: Integer,
    border_width = BORDER_WIDTH,
) {
    inner_gear = GearProfile(inner_teeth);
    outer_gear = GearProfile(outer_teeth);
    prop radius = outer_gear.radius;
    prop shift = inner_gear.radius * 40%;

    sector = {
        disc = Circle(r = outer_gear.radius - border_width);
        sector = Sector(r = outer_gear.radius, start = 30°, end = 90°).rotate(60°);
        inner_disc = Circle(r = inner_gear.radius - border_width * 250%);
        ring = Ring(
            o_r = inner_gear.radius + border_width - TEETH_SIZE * 2,
            thickness = border_width,
        );

        (disc - (sector - inner_disc) - ring.translate(x = shift))
            .buffer(-TEETH_SIZE * 2)
            .buffer(TEETH_SIZE * 2);
    };

    outer_gear - sector - inner_gear.translate(x = shift);
}

part PencilHole(pencil_hole_diameter = 0.5mm, offset = 0.0mm) {
    Circle(d = pencil_hole_diameter)
        .extrude(THICKNESS, scale = (x = 300%, y = 300%))
        .translate(y = offset)
}

part PencilHoles(radius: Length, n = 4, start = 4, angle = 22.5°) {
    {
        PencilHole(offset = [start..n * 2 - 1] * radius / n / 2);
        PencilHole(offset = ([start..n * 2 - 1] + 0.5) * radius / n / 2).rotate(-angle);
        PencilHole(offset = ([start..n * 2 - 1] + 1 / 3) * radius / n / 2).rotate(angle);
    }.rotate(90°)
}

#[export = "spirograph_frame.stl"]
#[color = std::color::BLUE]
{
    outer_gear = GearProfile(180);
    size = outer_gear.radius * 200% + 3 * BORDER_WIDTH;
    frame = {
        border = RoundedRect(size, radius = BORDER_WIDTH * 200%);
        border - outer_gear;
    }.extrude(THICKNESS * 2);

    logo2 = Logo(size = BORDER_WIDTH * 85%)
        .extrude(THICKNESS)
        .translate(x = size * 35%, y = size * 43%, z = THICKNESS * 150%);

    url = Text("microcad.xyz", BORDER_WIDTH * 150%)
        .center()
        .extrude(THICKNESS)
        .translate(x = size * 32%, y = -size * 46%, z = THICKNESS * 150%);

    frame - logo2 - url;
}.translate(x = 0mm);

ring_1 = RingProfile(outer_teeth = 180, inner_teeth = 120);
ring_2 = RingProfile(outer_teeth = 120, inner_teeth = 72);
ring_3 = RingProfile(outer_teeth = 72, inner_teeth = 36);

#[export = "spirograph_ring_1.stl"]
#[color = std::color::GREEN]
{
    ring = ring_1.extrude(THICKNESS);
    pencil_holes = PencilHoles(ring_1.radius, n = 6, start = 6);
    ring - pencil_holes;
}.translate();

#[export = "spirograph_ring_2.stl"]
#[color = std::color::RED]
{
    ring = ring_2.extrude(THICKNESS);
    pencil_holes = PencilHoles(ring_2.radius, n = 5, start = 5);
    ring - pencil_holes;
}.translate(x = ring_1.shift);

#[export = "spirograph_ring_3.stl"]
#[color = std::color::YELLOW]
{
    ring = ring_3.extrude(THICKNESS);
    pencil_holes = PencilHoles(ring_3.radius);
    ring - pencil_holes;
}.translate(x = ring_2.shift + ring_1.shift);

gear = {
    profile = GearProfile(36);
    gear = profile.extrude(THICKNESS);
    pencil_holes = PencilHoles(profile.radius, start = 1, n = 2, 120°);

    gear - pencil_holes;
};

#[export = "spirograph_gear.stl"]
#[color = std::color::TEAL]
gear.translate(x = ring_1.shift + ring_2.shift + ring_3.shift);
