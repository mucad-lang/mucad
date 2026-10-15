// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Diagnostics are issues that are presented to user.

mod diagnostic;
mod diagnostics;
mod issue;
mod issue_diagnostic;

pub use diagnostic::Diagnostic;
pub use diagnostics::{DiagRenderOptions, Diagnostics};
pub use issue::{Issue, IssueList, PushIssue};

/// Trait to implement `.to_diagnostics()`.
pub trait ToDiagnostics {
    fn to_diagnostics(&self) -> Diagnostics;
}

impl<I: Issue + Clone> ToDiagnostics for IssueList<I> {
    fn to_diagnostics(&self) -> Diagnostics {
        let mut diags = Diagnostics::new();
        self.iter().for_each(|issue| {
            diags.push(issue.clone());
        });
        diags
    }
}

impl<I: Issue> IssueList<I> {
    /// Convert the issues into diagnostics.
    ///
    /// Unlike [`ToDiagnostics::to_diagnostics`], the issues need not be `Clone`.
    pub fn into_diagnostics(self) -> Diagnostics {
        let mut diags = Diagnostics::new();
        self.into_iter().for_each(|issue| diags.push(issue));
        diags
    }
}
