// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

/// A part called `Lid` with three parameters.
part Lid(thickness = 1.6mm, inner_diameter = 16.0cm, height = 20.0mm) {
    // Calculate the outer diameter
    outer_diameter = 2.0 * thickness + inner_diameter;

    // Create two cylinders, one for the outer and one for the inner
    outer = std::geo3d::Cylinder(d = outer_diameter, h = height);
    inner = std::geo3d::Cylinder(d = inner_diameter, h = height).translate(z = thickness);

    // Calculate the difference between two translated cylinders and output them
    outer - inner;
}

// `l` is the instance of the lid model
l = Lid();

l; // Instantiate the lid.
