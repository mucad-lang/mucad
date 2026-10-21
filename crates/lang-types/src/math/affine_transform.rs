// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language types and values.

use serde::{Deserialize, Serialize};

use crate::{Angle, Length, Mat3, Mat4, Scalar, Vec3, math};

/// Transformation matrix
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AffineTransform {
    /// Translation.
    Translation {
        x: Length,
        y: Length,
        z: Length,
    },
    /// Generic rotation via matrix.
    RotateMatrix {
        m: Mat3,
    },
    RotateAroundAxis {
        angle: Angle,
        vec: Vec3,
    },
    EulerRotateXYZ {
        x: Angle,
        y: Angle,
        z: Angle,
    },
    EulerRotateZYX {
        x: Angle,
        y: Angle,
        z: Angle,
    },
    /// Scale.
    Scale(Vec3), // Impl display
    /// Uniform scale.
    UniformScale(Scalar),
    /// An arbitrary transform.
    Arbitrary(Mat4),
}

impl AffineTransform {
    pub fn matrix(&self) -> Mat4 {
        use math::{FromFloat, IntoFloat, Mat4F};

        match self {
            AffineTransform::Translation { x, y, z } => {
                let v = Vec3::new(x.0, y.0, z.0).into_float();
                let mat_f = Mat4F::from_translation(v);
                Mat4::from_float(mat_f)
            }
            AffineTransform::RotateMatrix { m } => math::mat3_to_mat4(*m),
            AffineTransform::RotateAroundAxis { angle, vec } => {
                math::mat3_to_mat4(math::rotate_around_axis(*angle, vec.x, vec.y, vec.z))
            }
            AffineTransform::EulerRotateXYZ { x, y, z } => {
                math::mat3_to_mat4(math::rotate_xyz(*x, *y, *z))
            }
            AffineTransform::EulerRotateZYX { x, y, z } => {
                math::mat3_to_mat4(math::rotate_zyx(*x, *y, *z))
            }
            AffineTransform::Scale(v) => {
                let mat_f = Mat4F::from_nonuniform_scale(
                    v.x.into_float(),
                    v.y.into_float(),
                    v.z.into_float(),
                );
                Mat4::from_float(mat_f)
            }
            AffineTransform::UniformScale(s) => {
                let s_f = (*s).into_float();
                let mat_f = Mat4F::from_scale(s_f);
                Mat4::from_float(mat_f)
            }
            AffineTransform::Arbitrary(m) => *m,
        }
    }
}

// --- Display Implementation ---

fn display_angle(angle: &Angle) -> String {
    format!("{}rad", angle.0)
}

impl std::fmt::Display for AffineTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::math::float::FromFixed;
        match self {
            AffineTransform::Translation { x, y, z } => {
                write!(f, "Translate({x}, {y}, {z})")
            }
            AffineTransform::RotateMatrix { m } => {
                write!(
                    f,
                    "RotateMatrix({m:?})",
                    m = math::float::Mat3F::from_fixed(*m)
                )
            }
            AffineTransform::RotateAroundAxis { angle, vec } => {
                write!(
                    f,
                    "Rotate({} around ({}, {}, {}))",
                    display_angle(angle),
                    vec.x,
                    vec.y,
                    vec.z
                )
            }
            AffineTransform::EulerRotateXYZ { x, y, z } => {
                write!(
                    f,
                    "EulerXYZ(x: {}, y: {}, z: {})",
                    display_angle(x),
                    display_angle(y),
                    display_angle(z)
                )
            }
            AffineTransform::EulerRotateZYX { x, y, z } => {
                write!(
                    f,
                    "EulerZYX(z: {}, y: {}, x: {})",
                    display_angle(z),
                    display_angle(y),
                    display_angle(x)
                )
            }
            AffineTransform::Scale(v) => {
                write!(f, "Scale(({}, {}, {}))", v.x, v.y, v.z)
            }
            AffineTransform::UniformScale(s) => {
                write!(f, "UniformScale({})", s)
            }
            AffineTransform::Arbitrary(m) => {
                write!(f, "Mat4([{:?}, {:?}, {:?}, {:?}])", m.x, m.y, m.z, m.w)
            }
        }
    }
}

// --- Hash Implementation ---

impl std::hash::Hash for AffineTransform {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the enum variant discriminator first
        std::mem::discriminant(self).hash(state);

        match self {
            AffineTransform::Translation { x, y, z } => {
                x.hash(state);
                y.hash(state);
                z.hash(state);
            }
            AffineTransform::RotateMatrix { m } => {
                let slice: &[Scalar; 9] = m.as_ref();
                bytemuck::bytes_of(slice).hash(state);
            }
            AffineTransform::RotateAroundAxis { angle, vec } => {
                angle.0.hash(state);
                vec.hash(state);
            }
            AffineTransform::EulerRotateXYZ { x, y, z }
            | AffineTransform::EulerRotateZYX { x, y, z } => {
                x.0.hash(state);
                y.0.hash(state);
                z.0.hash(state);
            }
            AffineTransform::Scale(v) => {
                v.hash(state);
            }
            AffineTransform::UniformScale(s) => {
                s.hash(state);
            }
            AffineTransform::Arbitrary(m) => {
                let slice: &[Scalar; 16] = m.as_ref();
                bytemuck::bytes_of(slice).hash(state);
            }
        }
    }
}
