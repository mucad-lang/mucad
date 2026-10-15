// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The diagnostic of an issue.

use miette::{Diagnostic, LabeledSpan, Severity, SourceCode};

use crate::diag::Issue;

/// The diagnostic of an issue: the severity of the issue and its code (e.g. `eval::error`)
/// with the help and labels of its error, warning or info.
#[derive(Debug)]
pub(crate) struct IssueDiagnostic<I: Issue>(pub I);

impl<I: Issue> std::fmt::Display for IssueDiagnostic<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<I: Issue> std::error::Error for IssueDiagnostic<I> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl<I: Issue> Diagnostic for IssueDiagnostic<I> {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.code().or_else(|| self.0.diagnostic().code())
    }

    fn severity(&self) -> Option<Severity> {
        Some(Issue::severity(&self.0))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.diagnostic().help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.diagnostic().url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.0.diagnostic().source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.0.diagnostic().labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.0.diagnostic().related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.0.diagnostic().diagnostic_source()
    }
}
