// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Diagnostics are issues that are presented to user.

mod diagnostic;
mod diagnostics;
mod issue;

pub use diagnostic::Diagnostic;
pub use diagnostics::{DiagRenderOptions, Diagnostics};
pub use issue::{Issue, IssueList, PushIssue};

/// Trait to implement `.into_diagnostics()`.
pub trait ToDiagnostics {
    fn to_diagnostics(&self) -> Diagnostics;
}

impl<I: Issue + Clone + Into<miette::Report>> ToDiagnostics for IssueList<I> {
    fn to_diagnostics(&self) -> Diagnostics {
        let mut diags = Diagnostics::new();
        self.iter().for_each(|issue| {
            diags.push(issue.clone());
        });
        diags
    }
}
