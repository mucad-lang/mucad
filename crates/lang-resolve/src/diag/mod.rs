// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;

pub use error::{ResolveError, ResolveErrorKind};
use microcad_lang_base::{Identifier, Issue, SrcRef, SrcReferrer, element::Case};
use microcad_lang_lower::{LowerInfo, LowerIssue, LowerWarning};
use microcad_lang_parse::{ParseInfo, ParseIssue, ParseWarning};
use miette::{Diagnostic, Severity};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveWarning {
    #[error("{0}")]
    Parse(#[from] ParseWarning),
    #[error("{0}")]
    Lower(#[from] LowerWarning),

    /// A symbol name is not written in the case of its kind, e.g. `UPPER_SNAKE_CASE` for constants.
    #[error("`{name}` should be written in {expected}")]
    WrongCase {
        name: Identifier,
        expected: Case,
        #[label("Not in {expected}")]
        src_ref: SrcRef,
    },
}

impl SrcReferrer for ResolveWarning {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::WrongCase { src_ref, .. } => *src_ref,
            Self::Parse(_) | Self::Lower(_) => SrcRef::none(),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveInfo {
    #[error("{0}")]
    Parse(#[from] ParseInfo),
    #[error("{0}")]
    Lower(#[from] LowerInfo),
}

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveIssue {
    /// A parse error.
    #[error(transparent)]
    #[diagnostic(severity(Error), code(resolve::error))]
    Err(#[from] ResolveError),

    /// A warning from the parser.
    #[error(transparent)]
    #[diagnostic(severity(Warning), code(resolve::warning))]
    Warn(#[from] ResolveWarning),

    /// An info from the parser.
    #[error(transparent)]
    #[diagnostic(severity(Advice), code(resolve::info))]
    Info(#[from] ResolveInfo),
}

impl Issue for ResolveIssue {
    type Err = ResolveError;
    type Warn = ResolveWarning;
    type Info = ResolveInfo;

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

impl From<ParseIssue> for ResolveIssue {
    fn from(issue: ParseIssue) -> Self {
        match issue {
            ParseIssue::Err(err) => ResolveIssue::Err(err.into()),
            ParseIssue::Warn(warn) => ResolveIssue::Warn(warn.into()),
            ParseIssue::Info(info) => ResolveIssue::Info(info.into()),
        }
    }
}

impl From<LowerIssue> for ResolveIssue {
    fn from(issue: LowerIssue) -> Self {
        match issue {
            LowerIssue::Err(err) => ResolveIssue::Err(err.into()),
            LowerIssue::Warn(warn) => ResolveIssue::Warn(warn.into()),
            LowerIssue::Info(info) => ResolveIssue::Info(info.into()),
        }
    }
}

pub type ResolveResult<T> = Result<T, ResolveError>;

impl SrcReferrer for ResolveIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Err(err) => err.src_ref(),
            Self::Warn(warn) => warn.src_ref(),
            Self::Info(_) => SrcRef::none(),
        }
    }
}
