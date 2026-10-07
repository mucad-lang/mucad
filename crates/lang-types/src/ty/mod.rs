// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad syntax elements of types

mod function_type;
mod matrix_type;
mod ops;
mod quantity_type;
mod tuple_type;
mod r#type;
mod type_list;
mod unit;

pub use matrix_type::*;
use miette::Diagnostic;
pub use quantity_type::*;
pub use tuple_type::*;
pub use r#type::*;
pub use type_list::*;
pub use unit::*;

pub use function_type::{CallParameter, CallParameters, CallSignature, FunctionType};

use crate::{BinaryOperator, UnaryOperator};

use thiserror::Error;

#[derive(Error, Debug, Clone, Diagnostic)]
pub enum TypeError {
    #[error("Unknown unit: {0}")]
    UnknownUnit(String),

    #[error("Unknown type: {0}")]
    UnknownType(String),

    /// Matrix type with invalid dimensions
    #[error("Invalid matrix type: {0}")]
    InvalidMatrixType(String),
    /// Binary operator not supported for the type(s)
    #[error("Operator '{op}' is not supported for types '{lhs}' and '{rhs}")]
    UnsupportedBinaryOperator {
        op: BinaryOperator,
        lhs: Type,
        rhs: Type,
    },

    /// Unary operator not supported for the type
    #[error("Unary operator '{op}' is not supported for type '{ty}'")]
    UnsupportedUnaryOperator { op: UnaryOperator, ty: Type },

    #[error("Invalid type: {0}")]
    InvalidType(Type),

    #[error("Non matching call signature parameters: {a} != {b}")]
    NonMatchingSignatureParameterList { a: CallSignature, b: CallSignature },

    #[error("Non matching function types: {a} != {b}")]
    NonMatchingFunctionTypes { a: FunctionType, b: FunctionType },

    #[error("Incompatible types '{lhs}' and '{rhs}'")]
    BinaryOpIncompatibleTypes {
        lhs: Type,
        rhs: Type,
        op: BinaryOperator,
    },

    #[error("Expected list type, got {0}")]
    NoListType(Type),

    #[error("Tuple {ty} has no field `{field}`")]
    TupleHasNoField { field: String, ty: Type },
}

impl TypeError {
    pub(crate) fn binary_op(
        lhs: impl Into<Type>,
        rhs: impl Into<Type>,
        op: BinaryOperator,
    ) -> Self {
        Self::BinaryOpIncompatibleTypes {
            lhs: lhs.into(),
            rhs: rhs.into(),
            op,
        }
    }
}

pub type TypeResult = Result<Type, TypeError>;

/// Trait for structs and expressions that have a type
pub trait Ty {
    /// Return type
    fn ty(&self) -> Type;
}
