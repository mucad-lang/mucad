// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::ControlFlow;

use crate::{Ast, ParseContext, ast};
use mucad_lang_base::{Severity, SpanToSrcRef};

/// This visitor collects all comments (*not* including doc comments).
#[derive(Debug, derive_more::Deref, Default)]
pub struct CommentCollector<'a>(Vec<&'a ast::Comment>);

impl<'ast> ast::Visitor<'ast> for CommentCollector<'ast> {
    type BreakTy = ();

    fn visit_comment(&mut self, comment: &'ast ast::Comment) -> ControlFlow<Self::BreakTy> {
        self.0.push(comment);
        std::ops::ControlFlow::Continue(())
    }
}

/// Expected diagnostic
#[derive(Debug, PartialEq)]
pub struct ExpectedDiagnostic {
    /// `error`, `warning`, `advice`
    pub severity: Severity,
    /// Line
    pub line: u32,
    /// An optional error code
    pub code: Option<String>,
}

impl ExpectedDiagnostic {
    /// Parse the diagnostics from the comment string.
    ///
    /// // warn
    /// // warning: LowerDiag
    pub fn from_comment(comment: &str, line: u32) -> Option<Self> {
        // 1. Clean the comment (remove "//" and trim whitespace)
        let content = comment.trim_start_matches('/').trim();
        if content.is_empty() {
            return None;
        }

        // 2. Split into parts: "severity: code"
        let mut parts = content.splitn(2, ':');
        let severity_str = parts.next()?.to_lowercase();
        let code = parts.next().map(|s| s.trim().to_string());

        // 3. Map severity
        let severity = match severity_str.as_str() {
            "error" => Severity::Error,
            "warn" | "warning" => Severity::Warning,
            "advice" | "help" => Severity::Advice,
            _ => return None, // Ignore comments that aren't diagnostics
        };

        Some(Self {
            severity,
            line,
            code,
        })
    }
}

/// A list of expected diagnostics, sorted by line number
#[derive(Debug, derive_more::Deref, PartialEq)]
pub struct ExpectedDiagnostics(Vec<ExpectedDiagnostic>);

impl ExpectedDiagnostics {
    /// Create a new list of expected diagnostics, sorted by line number.
    pub fn new(mut diags: Vec<ExpectedDiagnostic>) -> Self {
        diags.sort_by_key(|diag| diag.line);
        Self(diags)
    }
}

/// Expected the diagnostics collector
pub struct ExpectedDiagnosticsCollector<'source> {
    parse_context: &'source ParseContext<'source>,
    diagnostics: Vec<ExpectedDiagnostic>,
}

impl<'source> ExpectedDiagnosticsCollector<'source> {
    /// Create new collector.
    pub fn new(parse_context: &'source ParseContext<'source>) -> Self {
        Self {
            parse_context,
            diagnostics: Default::default(),
        }
    }

    /// Get diagnostics.
    pub fn diagnostics(self) -> ExpectedDiagnostics {
        ExpectedDiagnostics::new(self.diagnostics)
    }
}

impl<'source, 'ast> ast::Visitor<'ast> for ExpectedDiagnosticsCollector<'source> {
    type BreakTy = ();

    fn visit_comment(&mut self, node: &'ast ast::Comment) -> std::ops::ControlFlow<Self::BreakTy> {
        match &node.inner {
            ast::CommentInner::SingleLine(comment) => {
                match self.parse_context.span_to_src_ref(&node.span).line() {
                    Some(line) => {
                        self.diagnostics
                            .extend(ExpectedDiagnostic::from_comment(comment, line));
                    }
                    None => unreachable!(),
                }
            }
            ast::CommentInner::MultiLine(_) => {}
        }

        std::ops::ControlFlow::Continue(())
    }
}

/// Collect the expected diagnostics in an AST
pub fn collect_expected_diagnostics<'source>(
    parse_context: &ParseContext<'source>,
    ast: &Ast,
) -> ExpectedDiagnostics {
    use crate::ast::visitor::Visit;

    let mut collector = ExpectedDiagnosticsCollector::new(parse_context);
    let _ = ast.tree().visit(&mut collector);
    collector.diagnostics()
}
