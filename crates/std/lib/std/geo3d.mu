// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Sphere definition
///
/// Examples:
/// * with radius `r`: `Sphere(r = 10.0mm);`
pub part Sphere(radius: Length) {
    init(diameter: Length) {
        radius = diameter / 2;
    }

    __mu::geo3d::Sphere(radius);
}

/// Cube definition
///
/// Examples:
/// * with size `s`: `Cube(s = 10.0mm);`
/// * with size `x`, `y`, and `z`: `Cube(size_x = 10.0mm, size_y = 5.0mm, size_z = 3.0mm);`
pub part Cube(size_x: Length, size_y: Length, size_z: Length) {
    init(size: Length) {
        size_x = size;
        size_y = size;
        size_z = size;
    }

    std::geo2d::Rect(width = size_x, height = size_y)
        .std::ops::extrude(height = size_z)
        .std::ops::translate(z = -size_z / 2.0);
}

/// Cylinder definition.
///
/// Examples:
/// * with radius `r` and height `h`: `Cylinder(r = 10.0mm, h = 5.0mm)`
pub part Cylinder(radius_bottom: Length, radius_top: Length, height: Length, offset: Length = 0.0mm) {
    /// Initialize by `radius` and `height`. Cylinder will be centered.
    init(radius: Length, height: Length) {
        radius_bottom = radius;
        radius_top = radius;
        offset = -height / 2;
    }

    /// Initialize by `radius`, `height` and `offset`.
    init(radius: Length, height: Length, offset: Length) {
        radius_bottom = radius;
        radius_top = radius;
    }

    /// Initialize by `radius`, `bottom` and `top`.
    init(radius: Length, bottom: Length, top: Length) {
        radius_bottom = radius;
        radius_top = radius;
        height = top - bottom;
        offset = bottom;
    }

    /// Initialize by `diameter` and `height`. Cylinder will be centered.
    init(diameter: Length, height: Length) {
        radius_bottom = diameter / 2.0;
        radius_top = diameter / 2.0;
        offset = -height / 2;
    }

    s = radius_top / radius_bottom;
    std::geo2d::Circle(radius = radius_bottom)
        .std::ops::extrude(height, scale = (x = s, y = s))
        .std::ops::translate(z = offset);
}

/// A torus.
///
/// Examples:
/// * `Torus(major_radius = 45.0mm, minor_radius = 5.0mm)`
pub part Torus(major_radius: Length, minor_radius: Length) {
    std::geo2d::Circle(radius = minor_radius)
        .std::ops::translate(x = major_radius)
        .std::ops::revolve();
}
