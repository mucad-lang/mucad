// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Lower diagnostics.

mod error;

use microcad_lang_base::{Issue, SrcRef, SrcReferrer};
use miette::{Diagnostic, Severity};
use thiserror::Error;

pub use error::LowerError;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum LowerWarning {}

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum LowerInfo {}

/// An issue occurred during parsing.
#[derive(Clone, Debug, Error, Diagnostic)]
pub enum LowerIssue {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Err(#[from] LowerError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Warn(#[from] LowerWarning),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Info(#[from] LowerInfo),
}

impl Issue for LowerIssue {
    type Err = LowerError;
    type Warn = LowerWarning;
    type Info = LowerInfo;

    fn severity(&self) -> Severity {
        match self {
            Self::Err(_) => Severity::Error,
            Self::Warn(_) => Severity::Warning,
            Self::Info(_) => Severity::Advice,
        }
    }

    fn diagnostic(&self) -> &dyn Diagnostic {
        match self {
            Self::Err(err) => err,
            Self::Warn(warn) => warn,
            Self::Info(info) => info,
        }
    }
}

impl SrcReferrer for LowerIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Err(err) => err.src_ref(),
            _ => SrcRef::none(),
        }
    }
}

pub type LowerResult<T> = Result<T, LowerError>;
