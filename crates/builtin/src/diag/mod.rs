// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;

pub use error::BuiltinError;
use microcad_lang_base::{Issue, SrcRef, SrcReferrer};
use microcad_lang_types::Value;
use miette::{Diagnostic, Severity};
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
    /// A custom info message.
    #[error("{0}")]
    #[diagnostic(severity(Advice))]
    Custom(String),
}

/// An issue occurred during parsing.
#[derive(Clone, Debug, Error, Diagnostic)]
pub enum BuiltinIssue {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Err(#[from] BuiltinError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Warn(#[from] BuiltinWarning),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Info(#[from] BuiltinAdvice),
}

impl Issue for BuiltinIssue {
    type Err = BuiltinError;
    type Warn = BuiltinWarning;
    type Info = BuiltinAdvice;

    fn severity(&self) -> Severity {
        match self {
            Self::Err(_) => Severity::Error,
            Self::Warn(_) => Severity::Warning,
            Self::Info(_) => Severity::Advice,
        }
    }
}

impl SrcReferrer for BuiltinIssue {
    fn src_ref(&self) -> SrcRef {
        SrcRef::none()
    }
}

pub type BuiltinResult<T = Value> = Result<T, BuiltinError>;
