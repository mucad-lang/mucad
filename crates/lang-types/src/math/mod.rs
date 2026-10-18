// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mathematical operations.

mod affine_transform;
pub mod float;
mod ops;

pub use affine_transform::AffineTransform;
pub use float::*;
pub use ops::MathOps;

use crate::{Angle, Mat3, Mat4, Scalar, Vec3};

use cgmath::{InnerSpace, SquareMatrix};

/// Helper function to return rotation X,Y,Z rotation matrices.
fn rotation_matrices_xyz(x: Angle, y: Angle, z: Angle) -> (Mat3F, Mat3F, Mat3F) {
    (
        Mat3F::from_angle_x(AngleF::from_fixed(x)),
        Mat3F::from_angle_y(AngleF::from_fixed(y)),
        Mat3F::from_angle_z(AngleF::from_fixed(z)),
    )
}

pub fn rotate_around_axis(angle: Angle, x: Scalar, y: Scalar, z: Scalar) -> Mat3 {
    let axis = Vec3::new(x, y, z);
    let angle: AngleF = angle.into_float();
    Mat3F::from_axis_angle(axis.into_float(), angle).into_fixed()
}

pub fn rotate_xyz(x: Angle, y: Angle, z: Angle) -> Mat3 {
    let (x, y, z) = rotation_matrices_xyz(x, y, z);
    (x * y * z).into_fixed()
}

pub fn rotate_zyx(z: Angle, y: Angle, x: Angle) -> Mat3 {
    let (x, y, z) = rotation_matrices_xyz(x, y, z);
    (z * y * x).into_fixed()
}

/// Rotation matrix to orient a vector
pub fn orient_to_z(target: Vec3) -> Mat3 {
    use crate::math::Vec3F;

    let z_axis = Vec3F::unit_z();
    let target = Vec3F::from_fixed(target).normalize();

    // Handle edge case where target is already Z
    if (target - z_axis).magnitude2() < 1e-6 {
        return Mat3::from_float(Mat3F::identity());
    }

    // Handle 180-degree rotation (target is -Z)
    if (target + z_axis).magnitude2() < 1e-6 {
        // Rotate 180° around any axis perpendicular to Z.
        // For stability, pick X if possible, otherwise Y.
        let perp_axis = if z_axis.cross(Vec3F::unit_x()).magnitude2() > 1e-6 {
            Vec3F::unit_x()
        } else {
            Vec3F::unit_y()
        };
        return Mat3::from_float(Mat3F::from_axis_angle(
            perp_axis,
            cgmath::Rad(std::f64::consts::PI),
        ));
    }

    // Normal case
    let rotation_axis = z_axis.cross(target).normalize();
    let dot = z_axis.dot(target).clamp(-1.0, 1.0); // avoid NaNs
    let angle = cgmath::Rad(dot.acos());

    Mat3F::from_axis_angle(rotation_axis, angle).into_fixed()
}

pub fn align_vectors_rodrigues(from: Vec3, to: Vec3) -> Mat3 {
    use crate::math::Vec3F;

    let from = Vec3F::from_fixed(from).normalize();
    let to = Vec3F::from_fixed(to).normalize();

    let c = from.dot(to);

    if c > 1.0 - 1e-6 {
        return Mat3F::identity().into_fixed();
    }
    if c < -1.0 + 1e-6 {
        let perp_axis = if from.x.abs() > 0.9 {
            Vec3F::unit_y()
        } else {
            Vec3F::unit_x()
        };
        let axis = from.cross(perp_axis).normalize();
        return Mat3F::from_axis_angle(axis, cgmath::Rad(std::f64::consts::PI)).into_fixed();
    }

    let v = from.cross(to);

    // Skew-symmetric cross-product matrix [v]x
    let vx = Mat3F::new(0.0, v.z, -v.y, -v.z, 0.0, v.x, v.y, -v.x, 0.0);

    // R = I + vx + vx^2 * (1 / (1 + c))
    let k = 1.0 / (1.0 + c);
    (Mat3F::identity() + vx + (vx * vx) * k).into_fixed()
}

pub fn mat3_to_mat4(mat3: Mat3) -> Mat4 {
    let zero = Scalar::from_num(0.0);
    let one = Scalar::from_num(1.0);
    Mat4::new(
        mat3.x.x, mat3.x.y, mat3.x.z, zero, mat3.y.x, mat3.y.y, mat3.y.z, zero, mat3.z.x, mat3.z.y,
        mat3.z.z, zero, zero, zero, zero, one,
    )
}
