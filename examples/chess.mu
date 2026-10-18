// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later


use std::geo3d::*;
use std::ops::*;
use std::math::*;

// https://de.wikipedia.org/wiki/Bauhaus-Schachspiel

pub const TILE_SIZE = 5.8cm;
const BASE_SIZE = TILE_SIZE * 70%;

pub part Pawn() {
    Cube(BASE_SIZE * 80%)
}

pub part King() {
    {
        Cube(BASE_SIZE);
        Cube(BASE_SIZE / sqrt(2)).rotate(45°);
    }.align(Z)
}

pub part Queen() {
    {
        Cube(BASE_SIZE);
        Sphere(diameter = BASE_SIZE / sqrt(2));
    }.align(Z)
}

pub part Bishop() {
    size = BASE_SIZE / 3;
    Rect(size * 90%)
        .translate(size * 150%, [45, -135]°)
        .hull()
        .mirror(X)
        .union()
        .extrude(BASE_SIZE)
        .rotate(x = 90°)
        .center()
}

pub part Rook() {
    Cube(BASE_SIZE)
}

pub part Knight() {
    {
        height = BASE_SIZE / 2;
        r = Rect(BASE_SIZE);
        (r - r.translate(x = height, y = height)).extrude(height);
    }.rotate([0, 90]°).align(Z)
}

{
    Rook();
    Knight();
    Bishop();
    Queen();
    King();
    Bishop();
    Knight().rotate(x = 180°).rotate(90°);
    Rook();
    8 * Pawn();
}
.distribute_grid(cell_size = TILE_SIZE, rows = 2, columns = 8)
.translate(y = TILE_SIZE * 3)
.rotate([0, 180]°);

board_height = 1mm;
Rect(TILE_SIZE * 8).extrude(board_height).translate(z = -board_height);
