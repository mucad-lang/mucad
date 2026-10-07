// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Matrix value type

use crate::{MatrixType, Scalar, Ty, Type};

use serde::{Deserialize, Serialize};

/// 2D matrix type.
pub type Mat2 = cgmath::Matrix2<Scalar>;
/// 3D matrix type.
pub type Mat3 = cgmath::Matrix3<Scalar>;
/// 4D matrix type.
pub type Mat4 = cgmath::Matrix4<Scalar>;

/// Matrix type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Matrix {
    /// 2x2 matrix.
    Matrix2(Mat2),
    /// 3x3 matrix.
    Matrix3(Mat3),
    /// 4x4 matrix.
    Matrix4(Mat4),
}

impl Ty for Matrix {
    fn ty(&self) -> Type {
        match self {
            Matrix::Matrix2(_) => Type::Matrix(MatrixType::new(2, 2)),
            Matrix::Matrix3(_) => Type::Matrix(MatrixType::new(3, 3)),
            Matrix::Matrix4(_) => Type::Matrix(MatrixType::new(4, 4)),
        }
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Matrix::Matrix2(matrix2) => write!(f, "{matrix2:?}"),
            Matrix::Matrix3(matrix3) => write!(f, "{matrix3:?}"),
            Matrix::Matrix4(matrix4) => write!(f, "{matrix4:?}"),
        }
    }
}

impl std::hash::Hash for Matrix {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Matrix::Matrix2(matrix2) => {
                let slice: &[Scalar; 4] = matrix2.as_ref();
                bytemuck::bytes_of(slice).hash(state);
            }
            Matrix::Matrix3(matrix3) => {
                let slice: &[Scalar; 9] = matrix3.as_ref();
                bytemuck::bytes_of(slice).hash(state);
            }
            Matrix::Matrix4(matrix4) => {
                let slice: &[Scalar; 16] = matrix4.as_ref();
                bytemuck::bytes_of(slice).hash(state);
            }
        }
    }
}
