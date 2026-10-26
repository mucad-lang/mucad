// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Lower diagnostics.

mod error;

use miette::Diagnostic;
use mucad_lang_base::{SrcRef, SrcReferrer};
use mucad_macros::Issue;
use thiserror::Error;

pub use error::LowerError;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum LowerWarning {}

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum LowerAdvice {}

/// An issue occurred during parsing.
#[derive(Clone, Debug, Error, Issue)]
#[issue(code = "lower")]
pub enum LowerIssue {
    #[error(transparent)]
    Error(#[from] LowerError),

    #[error(transparent)]
    Warning(#[from] LowerWarning),

    #[error(transparent)]
    Advice(#[from] LowerAdvice),
}

impl SrcReferrer for LowerIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Error(err) => err.src_ref(),
            _ => SrcRef::none(),
        }
    }
}

pub type LowerResult<T> = Result<T, LowerError>;
