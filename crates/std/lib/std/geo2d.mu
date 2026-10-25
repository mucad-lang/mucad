// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

/// Circle definition with an offset.
///
/// Examples:
/// * With radius `r`: `Circle(r = 10.0mm);`
/// * With diameter `d`: `Circle(d = 5.0mm);`
pub sketch Circle(radius: Length, center = (x = 0.0mm, y = 0.0mm)) {
    init(diameter: Length, center = (x = 0.0mm, y = 0.0mm)) {
        radius = diameter / 2.0;
    }

    __mu::geo2d::Circle(radius)
        .__mu::ops::translate(x = center.x, y = center.y, z = 0mm);
}

/// A frame with a thickness.
pub sketch Frame(width: Length, height: Length, thickness: Length) {
    std::assert(thickness > 0.0mm, "Thickness must be a positive number (thickness = {thickness})");
    Rect(width, height) - Rect(width = width - 2*thickness, height = height - 2*thickness);
}

/// A regular hexagon.
pub sketch Hexagon(radius: Length) {
    Ngon(6, radius);
}


pub sketch InvoluteGearProfile(module: Length, teeth: Integer, pressure_angle = 0° /* The built-in formula is wrong, this should be 20° */) {
    prop pitch_radius = module * teeth / 2.0;
    prop outer_radius = pitch_radius + module;
    prop tooth_angle = 360° / teeth;
    prop tooth_distance = (outer_radius - pitch_radius) * __mu::math::PI;

    __mu::geo2d::InvoluteGearProfile(module, teeth, pressure_angle);
}


pub sketch Line(p0: Vec2, p1: Vec2) {
    init(width: Length, y = 0.0mm) {
        p0 = (x = -width / 2mm, y = y / 1mm);
        p1 = (x =  width / 2mm , y = y / 1mm);
    }

    init(height: Length, x = 0.0mm) {
        p0 = (x = x / 1mm, y = -height / 2mm);
        p1 = (x = x / 1mm, y =  height / 2mm);
    }

    init(width: Length, p: (x: Length, y: Length)) {
        p0 = (x = p.x / 1mm, y = p.y / 1mm);
        p1 = (x = (p.x + width) / 1mm , y = p.y / 1mm);
    }

    __mu::geo2d::Line(x0 = p0.x, y0 = p0.y, x1 = p1.x, y1 = p1.y);
}

/// A regular convex polygon with `n` corners.
pub sketch Ngon(n: Integer, radius: Length) {
    std::assert(n > 2, "N must be at least 3.");
    __mu::geo2d::Ngon(n, radius);
}

/// Rectangle definition.
///
/// # Examples
/// * Centered rect with width and height: `Rect(width = 10.0mm, height = 5.0mm);`
pub sketch Rect(width: Length, height: Length, x: Length, y: Length) {
    init(width: Length, height: Length) {
        x = -width / 2.0; // center x
        y = -height / 2.0; // center y
    }

    init(size: Length) {
        width = size;
        height = size;
        x = -size / 2.0; // center x
        y = -size / 2.0; // center y
    }

    __mu::geo2d::Rect(width, height, x, y);
}

pub sketch Ring(outer_radius: Length, inner_radius: Length ) {
     init(outer_radius: Length, thickness: Length) {
         inner_radius = outer_radius - thickness;
     }

     init(outer_diameter: Length, inner_diameter: Length ) {
         outer_radius = outer_diameter/2;
         inner_radius = inner_diameter/2;
     }

     init(outer_diameter: Length, thickness: Length) {
         inner_radius = outer_diameter/2 - thickness;
         outer_radius = outer_diameter/2;
     }

     Circle(radius = outer_radius) - Circle(radius = inner_radius);
}

/// A rounded rectangle.
///
/// # Examples
/// * Centered rect with width and height: `RoundedRect(width = 10.0mm, height = 5.0mm);`
pub sketch RoundedRect(
    width: Length,
    height: Length,
    x: Length,
    y: Length,
    radius_bottom_left: Length,
    radius_bottom_right: Length,
    radius_top_left: Length,
    radius_top_right: Length
) {
    init(width: Length, height: Length, radius: Length) {
        x = -width / 2.0; // center x
        y = -height / 2.0; // center y
        radius_bottom_left = radius;
        radius_bottom_right = radius;
        radius_top_left = radius;
        radius_top_right = radius;
    }

    init(size: Length, radius: Length) {
        width = size;
        height = size;
        x = -size / 2.0; // center x
        y = -size / 2.0; // center y
        radius_bottom_left = radius;
        radius_bottom_right = radius;
        radius_top_left = radius;
        radius_top_right = radius;
    }

    __mu::geo2d::RoundedRect(
        width, height, x, y,
        radius_bottom_left,
        radius_bottom_right,
        radius_top_left,
        radius_top_right
    );
}

/// Circle sector definition.
pub sketch Sector(radius: Length, start: Angle, end: Angle) {
    init(radius: Length, angle: Angle, offset = 0°) {
        start = offset - angle / 2;
        end = offset + angle / 2;
    }

    __mu::geo2d::Sector(radius, start, end);
}

pub sketch SinusoidalGearProfile(module: Length, teeth: Integer, roundness = 0.5, stretch = 0.5) {
    prop pitch_radius = module * teeth;
    prop outer_radius = pitch_radius + module;
    prop tooth_angle = 360° / teeth;

    __mu::geo2d::SinusoidalGearProfile(module, teeth, roundness, stretch);
}


/// Text definition.
pub sketch Text(height: Length, text: String, font_file = "") {
    __mu::geo2d::Text(height, text, font_file);
}
