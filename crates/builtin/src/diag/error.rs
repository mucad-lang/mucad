// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad built-in errors.

use microcad_lang_base::{BuiltinId, SrcRef, SrcReferrer};
use microcad_lang_types::{TypeError, ValueError};
use miette::Diagnostic;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error, Clone, Diagnostic)]
pub enum BuiltinError {
    #[error(transparent)]
    ValueError(#[from] ValueError),

    #[error(transparent)]
    TypeError(#[from] TypeError),

    #[error("No built-in `{full_name}` with '{id}'")]
    NoBuiltin { full_name: String, id: BuiltinId },

    #[error("Format error: {0}")]
    FormatError(#[from] std::fmt::Error),

    #[error("Builtin error in '{name}': {message}")]
    ExecutionFailed { name: String, message: String },

    #[error("Assertion failed: {message}")]
    AssertionFailed {
        message: String,
        /// The call of the assertion.
        src_ref: SrcRef,
    },

    #[error(
        "list index out of bounds: index is {index}, but list length is {len} (valid indices: 0..{len})"
    )]
    BadListIndex { index: usize, len: usize },

    #[error("Evaluation aborted. Panic: {0}")]
    Panic(String),

    #[error("Expected {message}")]
    Expected {
        message: String,
        /// The call of the expectation.
        src_ref: SrcRef,
    },

    #[error("Property not found: {name}")]
    PropertyNotFound { name: String },

    #[error("Element mismatch: {expected} != {actual}")]
    ElementMismatch { expected: String, actual: String },

    /// A custom error message.
    #[error("{0}")]
    #[diagnostic(severity(Error))]
    Custom(String),
}

impl SrcReferrer for BuiltinError {
    fn src_ref(&self) -> SrcRef {
        match self {
            BuiltinError::AssertionFailed { src_ref, .. }
            | BuiltinError::Expected { src_ref, .. } => *src_ref,
            _ => SrcRef::none(),
        }
    }
}
