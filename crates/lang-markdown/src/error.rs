// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Markdown parse errors.

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ParseError {
    /// A µcad code block without closing fence.
    #[error("Code block opened in line {line} is not closed")]
    #[diagnostic(help("Everything up to the end of the file is read as code."))]
    UnclosedCodeBlock { line: usize },

    #[error("Malformed header")]
    MalformedHeader,

    #[error("Invalid test result: {0}")]
    InvalidTestResult(String),

    #[error("Duplicated code block name: {0}")]
    DuplicatedCodeBlockName(String),
}
