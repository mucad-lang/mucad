// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{GetSourceByHash, SrcReferrer, diag::*};

use miette::Severity;
use std::io::IsTerminal;

/// Options that control the rendering of diagnostics
#[derive(Debug)]
pub struct DiagRenderOptions {
    /// Render diagnostic with colors
    pub color: bool,
    /// Render diagnostic with unicode characters
    pub unicode: bool,
}

impl DiagRenderOptions {
    /// Get the miette theme for the options
    pub fn theme(&self) -> miette::GraphicalTheme {
        match (self.unicode, self.color) {
            (true, true) => miette::GraphicalTheme::unicode(),
            (true, false) => miette::GraphicalTheme::unicode_nocolor(),
            (false, true) => miette::GraphicalTheme::ascii(),
            (false, false) => miette::GraphicalTheme::none(),
        }
    }
}

impl Default for DiagRenderOptions {
    fn default() -> Self {
        Self {
            color: std::env::var("NO_COLOR").as_deref().unwrap_or("0") == "0",
            unicode: std::io::stdout().is_terminal() && std::io::stderr().is_terminal(),
        }
    }
}

/// µcad source diagnostics.
#[derive(Debug, Default)]
pub struct Diagnostics {
    /// The number of overall errors in the evaluation process.
    error_count: u32,
    /// The number of overall warnings in the evaluation process.
    warning_count: u32,
    /// The list of diagnostics
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// Clear diagnostics.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }

    /// Merges another Diagnostics collection into this one.
    pub fn append(&mut self, mut other: Self) {
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        self.diagnostics.append(&mut other.diagnostics);
    }

    /// Return overall number of occurred errors.
    pub fn warning_count(&self) -> u32 {
        self.warning_count
    }

    /// Return overall number of occurred errors.
    pub fn error_count(&self) -> u32 {
        self.error_count
    }

    /// return lines with errors
    pub fn error_lines(&self) -> Vec<u32> {
        let mut lines: Vec<u32> = self
            .iter()
            .filter_map(|d| {
                if d.severity().unwrap_or_default() == Severity::Error {
                    d.src_ref.line()
                } else {
                    None
                }
            })
            .collect();
        lines.sort();
        lines.dedup();
        lines
    }

    /// return lines with warnings
    pub fn warning_lines(&self) -> Vec<u32> {
        let mut lines: Vec<u32> = self
            .iter()
            .filter_map(|d| {
                if d.severity().unwrap_or_default() == Severity::Warning {
                    d.src_ref.line()
                } else {
                    None
                }
            })
            .collect();
        lines.sort();
        lines.dedup();
        lines
    }

    pub fn push<I: Issue + SrcReferrer + Into<miette::Report>>(&mut self, issue: I) {
        let src_ref = issue.src_ref(); // Extract the metadata
        let report = issue.into(); // Convert to miette::Report

        match report.severity() {
            Some(Severity::Error) | None => self.error_count += 1,
            Some(Severity::Warning) => self.warning_count += 1,
            _ => {}
        };

        self.diagnostics.push(Diagnostic { report, src_ref });
    }

    pub fn render(
        &self,
        f: &mut dyn std::fmt::Write,
        source_by_hash: &impl GetSourceByHash,
        options: &DiagRenderOptions,
    ) -> std::fmt::Result {
        self.iter()
            .try_for_each(|diag| diag.render(f, source_by_hash, options))
    }

    pub fn render_to_string(
        &self,
        source_by_hash: &impl GetSourceByHash,
        options: &DiagRenderOptions,
    ) -> Result<String, std::fmt::Error> {
        let mut buffer = String::new();
        self.render(&mut buffer, source_by_hash, options)?;
        Ok(buffer)
    }

    /// Return `true` if `IssueList` is empty.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl<I: Issue> From<IssueList<I>> for Diagnostics {
    fn from(issues: IssueList<I>) -> Self {
        let mut diagnostics = Diagnostics::new();
        issues.into_iter().for_each(|issue| diagnostics.push(issue));
        diagnostics
    }
}
