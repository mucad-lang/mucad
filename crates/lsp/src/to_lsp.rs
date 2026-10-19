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

/// The byte offset of an LSP position in `code`, see [`position`].
///
/// A position after the end of its line is at the end of the line.
pub fn offset(code: &str, position: lsp::Position) -> usize {
    let line_start: usize = code
        .split_inclusive('\n')
        .take(position.line as usize)
        .map(str::len)
        .sum();
    let mut units = 0;
    for (index, ch) in code[line_start..].char_indices() {
        if units >= position.character || ch == '\n' {
            return line_start + index;
        }
        units += ch.len_utf16() as u32;
    }
    code.len()
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

/// The LSP completion item of a symbol.
pub fn completion_item(completion: &mu::resolve::Completion) -> lsp::CompletionItem {
    use mu::{base::element::WorkbenchKind, resolve::CompletionKind};

    let (kind, detail) = match completion.kind {
        CompletionKind::Library => (lsp::CompletionItemKind::MODULE, "library".into()),
        CompletionKind::Module => (lsp::CompletionItemKind::MODULE, "mod".into()),
        CompletionKind::Workbench(kind @ WorkbenchKind::Op) => {
            (lsp::CompletionItemKind::METHOD, kind.to_string())
        }
        CompletionKind::Workbench(kind) => (lsp::CompletionItemKind::CLASS, kind.to_string()),
        CompletionKind::Function => (lsp::CompletionItemKind::FUNCTION, "fn".into()),
        CompletionKind::Constant => (lsp::CompletionItemKind::CONSTANT, "const".into()),
        CompletionKind::Builtin => (lsp::CompletionItemKind::FUNCTION, "built-in".into()),
    };
    lsp::CompletionItem {
        label: completion.name.to_string(),
        kind: Some(kind),
        detail: Some(detail),
        documentation: completion.doc.clone().map(|doc| {
            lsp::Documentation::MarkupContent(lsp::MarkupContent {
                kind: lsp::MarkupKind::Markdown,
                value: doc,
            })
        }),
        ..Default::default()
    }
}
