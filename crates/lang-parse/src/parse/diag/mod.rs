// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;
mod rich;

pub(crate) use rich::{Rich, RichError, RichPattern, RichReason};

pub use error::{ParseError, ParseErrorKind};
use microcad_lang_base::{Issue, SrcRef, SrcReferrer};
use miette::{Diagnostic, Severity};
use thiserror::Error;

/// Warnings that occurred during parsing
#[derive(Clone, Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ParseWarning {}

/// Info and advice that occurred during parsing
#[derive(Clone, Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ParseInfo {}

/// An issue occurred during parsing.
#[derive(Clone, Debug, Error, Diagnostic)]
pub enum ParseIssue {
    /// A parse error.
    #[error(transparent)]
    #[diagnostic(severity(Error))]
    Err(#[from] ParseError),

    /// A warning from the parser.
    #[error(transparent)]
    #[diagnostic(severity(Warning))]
    Warn(#[from] ParseWarning),

    /// An info from the parser.
    #[error(transparent)]
    #[diagnostic(severity(Advice))]
    Info(#[from] ParseInfo),
}

impl Issue for ParseIssue {
    type Err = ParseError;
    type Warn = ParseWarning;
    type Info = ParseInfo;

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

impl SrcReferrer for ParseIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Err(err) => err.src_ref,
            _ => SrcRef::none(),
        }
    }
}
