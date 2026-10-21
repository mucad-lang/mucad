// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub use __mu::ops::center;
pub use __mu::ops::hull;
pub use __mu::ops::intersect;
pub use __mu::ops::subtract;
pub use __mu::ops::multiply;
pub use __mu::ops::union;

/// Align a geometry collection along an axis with a certain spacing.
pub op align(direction: Vec3, spacing = 0mm) {
    @input.__mu::ops::align(
        x = direction.x,
        y = direction.y,
        z = direction.z,
        spacing
    );
}

/// Create a new geometry whose boundary is offset the specified distance from the input.
pub op buffer(distance: Length) {
    @input.__mu::ops::buffer(distance);
}

/// Generate a contour geometry out of an input geometry.
pub op contour(thickness: Length, distance = 0mm) {
    @input.buffer(distance + thickness / 2) - @input.buffer(distance - thickness / 2);
}


/// Distribute geometries within a 2D rectangular grid.
///
/// The `distribute_grid` operator arranges elements evenly inside a rectangular region,
/// subdivided into a specified number of rows and columns. Each grid cell receives
/// one element, allowing for structured placement of objects, points, or geometries.
///
/// The grid is defined by its position (`x`, `y`), total dimensions (`width`, `height`),
/// and subdivision counts (`rows`, `columns`).
pub op distribute_grid(x: Length, y: Length, width: Length, height: Length, rows: Integer, columns: Integer) {
    /// Initialize a centered rectangular grid.
    init(width: Length, height: Length, rows: Integer, columns: Integer) {
        x = -width / 2.0; // center x
        y = -height / 2.0; // center y
    }

    /// Initialize a centered square grid.
    ///
    /// The `init` helper simplifies grid creation by specifying a single `size`
    /// value instead of separate width and height. It automatically centers
    /// the grid around the origin and defines its overall dimensions.
    init(size: Length, rows: Integer, columns: Integer) {
        width = size;
        height = size;
        x = -size / 2.0; // center x
        y = -size / 2.0; // center y
    }

    /// Initialize a centered grid with a squared cell size.
    ///
    /// The `init` helper simplifies grid creation by specifying a single `cell_size`
    /// value instead of separate width and height. It automatically centers
    /// the grid around the origin and defines its overall dimensions.
    init(cell_size: Length, rows: Integer, columns: Integer) {
        width = cell_size * columns;
        height = cell_size * rows;
        x = -width / 2.0; // center x
        y = -height / 2.0; // center y
    }


    @input.__mu::ops::distribute_grid(x, y, width, height, rows, columns);
}

/// Linear extrude a 2D geometry into a 3D.
pub op extrude(
        height: Length,
        scale = (x = 100%, y = 100%),
        twist = 0°,
) {
    @input.__mu::ops::extrude(height, scale_x = scale.x, scale_y = scale.y, twist);
}

/// Mirror along an axis, keeping existing geometry.
pub op mirror(x = 0.0mm, y = 0.0mm, z = 0.0mm, n: Vec3) {
    @input.__mu::ops::mirror(
        x, y, z, nx = n.x, ny = n.y, nz = n.z
    );
}

/// Orient an object towards a vector.
pub op orient(v: Vec3) {
    @input.__mu::ops::orient(x = v.x, y = v.y, z = v.z);
}

/// Reflect a geometry along an axis.
pub op reflect(x = 0.0mm, y = 0.0mm, z = 0.0mm, n: Vec3) {
    @input.__mu::ops::reflect(
        x, y, z, nx = n.x, ny = n.y, nz = n.z
    );
}

/// Revolve a 2D geometry about the Z axis, its Y axis becomes the Z axis.
///
/// A positive angle revolves counter-clockwise (right-hand rule), from the X towards the Y axis.
pub op revolve(angle = 360°) {
    @input.__mu::ops::revolve(angle);
}

/// An operation that rotates a part.
///
/// See the `init`s for examples on how to use this operation.
pub op rotate(matrix: Matrix3) {
    /// Rotate around an axis by an angle using a 3D vector.
    ///
    /// The `axis` vector will be normalized.
    ///
    /// * `angle` - An angle.
    /// * `axis` - An axis to rotate around (Z axis is default).
    ///
    /// Examples:
    /// * `Rect(42mm).rotate(45°);`: 2D rotation of a square by 45°.
    /// * `Cube(42mm).rotate(45°, (x = 1, y = 1, z = 1)) ;`: Rotates a cube by 45° around a diagonal axis.
    /// * `Cube(42mm).rotate(30°, Y);`: Rotates a cube by 30° around Y axis.
    init(angle: Angle, axis = __mu::math::Z) {
        matrix = __mu::math::rotate_around_axis(angle, x = axis.x, y = axis.y, z = axis.z);
    }

    /// Rotation about the X, then the Y, then the Z axis by Euler angles.
    ///
    /// The axes stay fixed, positive angles rotate counter-clockwise (right-hand rule).
    ///
    /// * `x` - X angle.
    /// * `y` - Y angle.
    /// * `z` - Z angle.
    ///
    /// Examples:
    /// * `Cube(42mm).rotate(x = 45°);`: Rotates a cube by 45° about the X axis.
    /// * `Cube(42mm).rotate(x = 90°, z = 90°);`: Rotates a cube about the X axis, then about the Z axis.
    init(x = 0°, y = 0°, z = 0°) {
        matrix = __mu::math::rotate_xyz(x = x, y = y, z = z);
    }

    /// Rotation by yaw, pitch and roll like in FreeCAD: rotates about the X axis by `roll`,
    /// then about the Y axis by `pitch`, then about the Z axis by `yaw`.
    ///
    /// The axes stay fixed, positive angles rotate counter-clockwise (right-hand rule).
    ///
    /// * `roll` - Roll angle, rotates about the X axis.
    /// * `pitch` - Pitch angle, rotates about the Y axis.
    /// * `yaw` - Yaw angle, rotates about the Z axis.
    ///
    /// Example:
    /// * `Cube(42mm).rotate(roll = 30°);`: Rotates a cube by 30° about the X axis.
    init(roll = 0°, pitch = 0°, yaw = 0°) {
        matrix = __mu::math::rotate_xyz(x = roll, y = pitch, z = yaw);
    }

    /// Rotation about the X, then the Y, then the Z axis (with a tuple of angles).
    ///
    /// * `Cube(42mm).rotate(xyz = (x = 30°, y = 20°, z = 10°));`: Rotates a cube about X, then Y, then Z.
    init(xyz = (x = 0°, y = 0°, z = 0°)) {
        matrix = __mu::math::rotate_xyz(x = xyz.x, y = xyz.y, z = xyz.z);
    }

    /// Rotation about the Z, then the Y, then the X axis (with a tuple of angles).
    ///
    /// * `Cube(42mm).rotate(zyx = (x = 30°, y = 20°, z = 10°));`: Rotates a cube about Z, then Y, then X.
    init(zyx = (x = 0°, y = 0°, z = 0°)) {
        matrix = __mu::math::rotate_zyx(x = zyx.x, y = zyx.y, z = zyx.z);
    }

    @input.__mu::ops::rotate(matrix);
}

/// An operation that scales a part.
///
/// See the `init`s for examples on how to use this operation.
pub op scale(v: Vec3) {
    /// Uniform scale.
    ///
    /// * `s` - Scale factor.
    ///
    /// Examples:
    /// * `Rect(42mm).scale(200%);`: Scale a rectangle by 200%.
    init(s: Scalar) {
        v = (x = s, y = s, z = s);
    }

    /// Non-uniform scale.
    ///
    /// * `x` - Scale factor in X direction.
    /// * `y` - Scale factor in Y direction.
    /// * `z` - Scale factor in Z direction.
    ///
    /// Examples:
    /// * `Rect(42mm).scale(x = 2.0);`: Scale a rectangle in X direction.
    init(x: Scalar = 1.0, y: Scalar = 1.0, z: Scalar = 1.0) {
        v = (x = x, y = y, z = z);
    }

    @input.__mu::ops::scale(x = v.x, y = v.y, z = v.z);
}

/// Create a spiral with a height and an inner and outer radius about the Z axis.
///
/// A positive angle makes a right-handed spiral, which turns counter-clockwise upwards.
pub op spiralize(
    inner_radius: Length,
    outer_radius: Length,
    angle: Angle,
    height = 0mm
) {
    @input.__mu::ops::spiralize(height, inner_radius, outer_radius, angle);
}

/// An operation that translates (moves) a geometry.
pub op translate(x = 0.0mm, y = 0.0mm, z = 0.0mm) {
    /// Polar-style translate with distance and angle.
    init(distance: Length, angle: Angle) {
        x = distance * __mu::math::cos(angle);
        y = distance * __mu::math::sin(angle);
        z = 0.0mm;
    }

    @input.__mu::ops::translate(x, y, z);
}
