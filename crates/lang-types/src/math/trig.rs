// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rotation matrices, which are the same on all platforms.

use super::{AngleF, Mat3F, Vec3F};

/// A rotation about the X axis by `angle`.
pub fn rotation_x(angle: AngleF) -> Mat3F {
    let (s, c) = libm::sincos(angle.0);
    Mat3F::new(1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c)
}

/// A rotation about the Y axis by `angle`.
pub fn rotation_y(angle: AngleF) -> Mat3F {
    let (s, c) = libm::sincos(angle.0);
    Mat3F::new(c, 0.0, -s, 0.0, 1.0, 0.0, s, 0.0, c)
}

/// A rotation about the Z axis by `angle`.
pub fn rotation_z(angle: AngleF) -> Mat3F {
    let (s, c) = libm::sincos(angle.0);
    Mat3F::new(c, s, 0.0, -s, c, 0.0, 0.0, 0.0, 1.0)
}

/// A rotation about `axis`, which is a unit vector, by `angle`.
pub fn rotation_axis(axis: Vec3F, angle: AngleF) -> Mat3F {
    let (s, c) = libm::sincos(angle.0);
    let one_minus_c = 1.0 - c;
    Mat3F::new(
        one_minus_c * axis.x * axis.x + c,
        one_minus_c * axis.x * axis.y + s * axis.z,
        one_minus_c * axis.x * axis.z - s * axis.y,
        one_minus_c * axis.x * axis.y - s * axis.z,
        one_minus_c * axis.y * axis.y + c,
        one_minus_c * axis.y * axis.z + s * axis.x,
        one_minus_c * axis.x * axis.z + s * axis.y,
        one_minus_c * axis.y * axis.z - s * axis.x,
        one_minus_c * axis.z * axis.z + c,
    )
}
