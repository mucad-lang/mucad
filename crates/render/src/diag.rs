// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render issues, which are collected while rendering continues.

use microcad_lang_base::{Issue, SrcRef, SrcReferrer};
use miette::{Diagnostic, Severity};
use thiserror::Error;

use crate::RenderError;

/// A warning that occurred during rendering.
#[derive(Debug, Error, Diagnostic)]
pub enum RenderWarning {}

/// An info that occurred during rendering.
#[derive(Debug, Error, Diagnostic)]
pub enum RenderInfo {}

/// An issue that occurred during rendering.
#[derive(Debug, Error, Diagnostic)]
pub enum RenderIssue {
    /// A render error.
    #[error(transparent)]
    #[diagnostic(code(render::error))]
    Err(#[from] RenderError),

    /// A render warning.
    #[error(transparent)]
    #[diagnostic(code(render::warning))]
    Warn(#[from] RenderWarning),

    /// A render info.
    #[error(transparent)]
    #[diagnostic(code(render::info))]
    Info(#[from] RenderInfo),
}

impl Issue for RenderIssue {
    type Err = RenderError;
    type Warn = RenderWarning;
    type Info = RenderInfo;

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

impl SrcReferrer for RenderIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Err(err) => err.src_ref(),
            Self::Warn(_) | Self::Info(_) => SrcRef::none(),
        }
    }
}
