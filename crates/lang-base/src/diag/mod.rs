// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Diagnostics are issues that are presented to user.

mod diagnostic;
mod diagnostics;
mod issue;

pub use diagnostic::Diagnostic;
pub use diagnostics::{DiagRenderOptions, Diagnostics};
pub use issue::{Issue, IssueList, PushIssue};

/// Render diagnostics as text, e.g. with the source code they refer to.
pub trait RenderDiag {
    /// Render `diagnostics` with `options` into `f`.
    fn render_diag_with(
        &self,
        f: &mut dyn std::fmt::Write,
        diagnostics: &Diagnostics,
        options: &DiagRenderOptions,
    ) -> std::fmt::Result;

    /// Render diagnostics into a string.
    fn render_diag_to_string(&self, diagnostics: &Diagnostics) -> String {
        self.render_diag_to_string_with(diagnostics, &DiagRenderOptions::default())
    }

    /// Render diagnostics with `options` into a string, e.g. without colors for a log file.
    fn render_diag_to_string_with(
        &self,
        diagnostics: &Diagnostics,
        options: &DiagRenderOptions,
    ) -> String {
        let mut buff = String::new();
        self.render_diag_with(&mut buff, diagnostics, options)
            .expect("format to string can't fail");
        buff
    }
}

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
