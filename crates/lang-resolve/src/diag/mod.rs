// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;

pub use error::{ResolveError, ResolveErrorKind};
use miette::Diagnostic;
use mucad_lang_base::{Identifier, SrcRef, SrcReferrer, element::Case};
use mucad_lang_lower::{LowerAdvice, LowerIssue, LowerWarning};
use mucad_lang_parse::{ParseAdvice, ParseIssue, ParseWarning};
use mucad_macros::Issue;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveWarning {
    #[error(transparent)]
    Parse(#[from] ParseWarning),
    #[error(transparent)]
    Lower(#[from] LowerWarning),

    /// A symbol name is not written in the case of its kind, e.g. `UPPER_SNAKE_CASE` for constants.
    #[error("`{name}` should be written in {expected}")]
    WrongCase {
        name: Identifier,
        expected: Case,
        #[label("Not in {expected}")]
        src_ref: SrcRef,
    },

    /// A local is never used, e.g. a parameter or variable of a function.
    #[error("Unused local `{name}`")]
    #[diagnostic(help("Remove it or start its name with `_`, e.g. `_{name}`"))]
    UnusedLocal {
        name: Identifier,
        #[label("Never used")]
        src_ref: SrcRef,
    },
}

impl SrcReferrer for ResolveWarning {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::WrongCase { src_ref, .. } | Self::UnusedLocal { src_ref, .. } => *src_ref,
            Self::Parse(_) | Self::Lower(_) => SrcRef::none(),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveAdvice {
    #[error(transparent)]
    Parse(#[from] ParseAdvice),
    #[error(transparent)]
    Lower(#[from] LowerAdvice),
}

#[derive(Debug, Error, Issue)]
#[issue(code = "resolve")]
pub enum ResolveIssue {
    /// A resolve error.
    #[error(transparent)]
    Error(#[from] ResolveError),

    /// A resolve warning.
    #[error(transparent)]
    Warning(#[from] ResolveWarning),

    /// A resolve advice.
    #[error(transparent)]
    Advice(#[from] ResolveAdvice),
}

impl From<ParseIssue> for ResolveIssue {
    fn from(issue: ParseIssue) -> Self {
        match issue {
            ParseIssue::Error(err) => ResolveIssue::Error(err.into()),
            ParseIssue::Warning(warn) => ResolveIssue::Warning(warn.into()),
            ParseIssue::Advice(advice) => ResolveIssue::Advice(advice.into()),
        }
    }
}

impl From<LowerIssue> for ResolveIssue {
    fn from(issue: LowerIssue) -> Self {
        match issue {
            LowerIssue::Error(err) => ResolveIssue::Error(err.into()),
            LowerIssue::Warning(warn) => ResolveIssue::Warning(warn.into()),
            LowerIssue::Advice(advice) => ResolveIssue::Advice(advice.into()),
        }
    }
}

pub type ResolveResult<T> = Result<T, ResolveError>;

impl SrcReferrer for ResolveIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Error(err) => err.src_ref(),
            Self::Warning(warn) => warn.src_ref(),
            Self::Advice(_) => SrcRef::none(),
        }
    }
}
