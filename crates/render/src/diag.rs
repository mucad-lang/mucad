// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render issues, which are collected while rendering continues.

use mucad_lang_base::{SrcRef, SrcReferrer};
use miette::Diagnostic;
use thiserror::Error;

use crate::RenderError;

/// A warning that occurred during rendering.
#[derive(Debug, Error, Diagnostic)]
pub enum RenderWarning {}

/// An advice that occurred during rendering.
#[derive(Debug, Error, Diagnostic)]
pub enum RenderAdvice {}

/// An issue that occurred during rendering.
#[derive(Debug, Error, mucad_macros::Issue)]
#[issue(code = "render")]
pub enum RenderIssue {
    /// A render error.
    #[error(transparent)]
    Error(#[from] RenderError),

    /// A render warning.
    #[error(transparent)]
    Warning(#[from] RenderWarning),

    /// A render advice.
    #[error(transparent)]
    Advice(#[from] RenderAdvice),
}

impl SrcReferrer for RenderIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Error(err) => err.src_ref(),
            Self::Warning(_) | Self::Advice(_) => SrcRef::none(),
        }
    }
}
