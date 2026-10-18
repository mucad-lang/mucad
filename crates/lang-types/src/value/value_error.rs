// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Value errors.

use miette::Diagnostic;
use thiserror::Error;

use mucad_lang_base::Identifier;

use crate::{Type, TypeError};

/// Value error
#[derive(Debug, Error, Clone, Diagnostic)]
pub enum ValueError {
    /// Type error
    #[error("Type error: {0}")]
    TypeError(#[from] TypeError),

    /// Invalid operator
    #[error("Invalid operator: {0}")]
    InvalidOperator(String),

    /// Cannot convert to color.
    #[error("Cannot convert named tuple to color: {0}")]
    CannotConvertToColor(String),

    /// Cannot add unit to a value that has already a unit
    #[error("Cannot add unit to a value that has already a unit: {0}")]
    CannotAddUnitToValueWithUnit(String),

    /// Cannot convert value
    #[error("Cannot convert value {0} to {1}")]
    CannotConvert(String, String),

    /// Cannot concat two vec with different types
    #[error("Cannot concat two vec with different types {0} and {1}")]
    CannotCombineVecOfDifferentType(Type, Type),

    /// Tuple length mismatch
    #[error("Tuple type mismatch: lhs={lhs}, rhs={rhs}")]
    TupleTypeMismatch {
        /// Left hand operand.
        lhs: Type,
        /// Right hand operand.
        rhs: Type,
    },

    /// Duplicate parameter
    #[error("Duplicate parameter: {0}")]
    DuplicateParameter(Identifier),

    /// Could not find identifier
    #[error("Identifier not found: {0}")]
    IdNotFound(Identifier),

    /// Expected a common type, e.g. for a [`ValueList`].
    #[error("Common type expected")]
    CommonTypeExpected,

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Invalid type: {0}")]
    InvalidMathOperand(Type),
}
