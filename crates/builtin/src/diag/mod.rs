// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;

pub use error::BuiltinError;
use microcad_lang_base::{SrcRef, SrcReferrer};
use microcad_lang_types::Value;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum BuiltinWarning {
    /// A custom warning message.
    #[error("{0}")]
    #[diagnostic(severity(Warning))]
    Custom(String),
}

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum BuiltinAdvice {
    /// A custom advice message.
    #[error("{0}")]
    #[diagnostic(severity(Advice))]
    Custom(String),
}

/// An issue occurred during parsing.
#[derive(Clone, Debug, Error, microcad_macros::Issue)]
#[issue(code = "builtin")]
pub enum BuiltinIssue {
    #[error(transparent)]
    Error(#[from] BuiltinError),

    #[error(transparent)]
    Warning(#[from] BuiltinWarning),

    #[error(transparent)]
    Advice(#[from] BuiltinAdvice),
}

impl SrcReferrer for BuiltinIssue {
    fn src_ref(&self) -> SrcRef {
        SrcRef::none()
    }
}

pub type BuiltinResult<T = Value> = Result<T, BuiltinError>;
