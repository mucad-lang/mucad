// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Various type conversion from µcad to tower_lsp's types.

use tower_lsp::lsp_types as lsp;

use mucad_driver::prelude as mu;

/// The LSP position of the byte `offset` in `code`.
///
/// Lines start at zero, columns are counted in UTF-16 code units, as LSP clients expect by default.
pub fn position(code: &str, offset: usize) -> lsp::Position {
    let before = &code[..offset.min(code.len())];
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    lsp::Position::new(
        before.matches('\n').count() as u32,
        before[line_start..].encode_utf16().count() as u32,
    )
}

/// The LSP range of a source reference into `code`.
pub fn range(code: &str, src_ref: &mu::SrcRef) -> lsp::Range {
    lsp::Range::new(position(code, src_ref.start), position(code, src_ref.end))
}

/// Conversion of µcad types, which refer to a source, into LSP types.
pub trait ToLsp {
    type Output;

    /// Convert into an LSP type, source references refer to `source`.
    fn to_lsp(&self, source: &mu::Source) -> Self::Output;
}

impl ToLsp for mu::Diagnostic {
    type Output = Option<lsp::Diagnostic>;

    /// A diagnostic in `source` or without a source reference, e.g. a missing standard library.
    ///
    /// Diagnostics in other sources, e.g. file modules, are skipped.
    fn to_lsp(&self, source: &mu::Source) -> Self::Output {
        let range = if self.src_ref.is_none() {
            lsp::Range::default()
        } else if self.src_ref.source_hash() == source.hash_id() {
            range(source.code(), &self.src_ref)
        } else {
            return None;
        };

        Some(lsp::Diagnostic {
            range,
            severity: Some(match self.severity() {
                Some(miette::Severity::Advice) => lsp::DiagnosticSeverity::HINT,
                Some(miette::Severity::Warning) => lsp::DiagnosticSeverity::WARNING,
                Some(miette::Severity::Error) | None => lsp::DiagnosticSeverity::ERROR,
            }),
            code: self
                .code()
                .map(|code| lsp::NumberOrString::String(code.to_string())),
            source: Some("µcad".into()),
            message: match self.help() {
                Some(help) => format!("{}\n{help}", self.report),
                None => self.report.to_string(),
            },
            ..Default::default()
        })
    }
}

impl ToLsp for mu::Diagnostics {
    type Output = lsp::FullDocumentDiagnosticReport;

    fn to_lsp(&self, source: &mu::Source) -> Self::Output {
        Self::Output {
            result_id: None,
            items: self.iter().filter_map(|diag| diag.to_lsp(source)).collect(),
        }
    }
}

impl ToLsp for mu::TextEdit {
    type Output = lsp::TextEdit;

    fn to_lsp(&self, source: &mu::Source) -> Self::Output {
        Self::Output {
            range: range(source.code(), &self.src_ref),
            new_text: self.new_text.clone(),
        }
    }
}
