// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;
mod rich;

pub(crate) use rich::{Rich, RichError, RichPattern, RichReason};

pub use error::{ParseError, ParseErrorKind};
use mucad_lang_base::{SrcRef, SrcReferrer};
use mucad_macros::Issue;
use miette::Diagnostic;
use thiserror::Error;

/// Warnings that occurred during parsing
#[derive(Clone, Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ParseWarning {}

/// Info and advice that occurred during parsing
#[derive(Clone, Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ParseAdvice {}

/// An issue occurred during parsing.
#[derive(Clone, Debug, Error, Issue)]
#[issue(code = "parse")]
pub enum ParseIssue {
    /// A parse error.
    #[error(transparent)]
    Error(#[from] ParseError),

    /// A warning from the parser.
    #[error(transparent)]
    Warning(#[from] ParseWarning),

    /// An advice from the parser.
    #[error(transparent)]
    Advice(#[from] ParseAdvice),
}

impl SrcReferrer for ParseIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Error(err) => err.src_ref,
            _ => SrcRef::none(),
        }
    }
}
