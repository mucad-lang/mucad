// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Boolean operations

#[derive(Debug, Clone, Copy)]
/// Boolean operations
pub enum BooleanOp {
    /// Computes the union R = P ∪ Q
    Union,
    /// computes the difference R = P ∖ Q
    Subtract,
    /// computes the complement R=P̅
    Complement,
    /// computes the intersection R = P ∩ Q
    Intersect,
}

pub use geo::OpType as OpType2D;
pub use manifold_rust::types::OpType as OpType3D;

impl From<BooleanOp> for OpType2D {
    fn from(op: BooleanOp) -> Self {
        match op {
            BooleanOp::Subtract => OpType2D::Difference,
            BooleanOp::Union => OpType2D::Union,
            BooleanOp::Intersect => OpType2D::Intersection,
            BooleanOp::Complement => OpType2D::Xor,
        }
    }
}

impl From<BooleanOp> for OpType3D {
    fn from(op: BooleanOp) -> Self {
        match op {
            BooleanOp::Subtract => OpType3D::Subtract,
            BooleanOp::Union => OpType3D::Add,
            BooleanOp::Intersect => OpType3D::Intersect,
            BooleanOp::Complement => unimplemented!(),
        }
    }
}

/// A trait for a unary boolean operation, used in geometry collections.
pub trait UnaryBooleanOp {
    /// The expected output type.
    type Output;

    /// Perform a boolean operation.
    fn boolean_op(&self, op: BooleanOp) -> Self::Output;

    /// Calculate the union.
    fn union(&self) -> Self::Output {
        self.boolean_op(BooleanOp::Union)
    }

    /// Calculate the difference.
    fn difference(&self) -> Self::Output {
        self.boolean_op(BooleanOp::Subtract)
    }

    /// Calculate the intersection.
    fn intersection(&self) -> Self::Output {
        self.boolean_op(BooleanOp::Intersect)
    }
}
