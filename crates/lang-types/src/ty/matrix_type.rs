// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Matrix type

use serde::{Deserialize, Serialize};

use crate::{BinaryOperator, TypeError, TypeResult};

/// M x N Matrix Type.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MatrixType {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub columns: usize,
}

impl MatrixType {
    /// Create new matrix type with rows and columns
    pub fn new(rows: usize, columns: usize) -> Self {
        Self { rows, columns }
    }
}

impl std::fmt::Display for MatrixType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Matrix{}x{}", self.rows, self.columns)
    }
}

impl std::ops::Add for MatrixType {
    type Output = TypeResult;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self;
        if lhs == rhs {
            Ok(lhs.into())
        } else {
            Err(TypeError::binary_op(lhs, rhs, BinaryOperator::Add))
        }
    }
}

impl std::ops::Sub for MatrixType {
    type Output = TypeResult;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self;
        if lhs == rhs {
            Ok(lhs.into())
        } else {
            Err(TypeError::binary_op(lhs, rhs, BinaryOperator::Subtract))
        }
    }
}
