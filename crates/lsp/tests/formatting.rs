// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test document formatting with the LSP service.

mod common;

use mucad_driver::prelude as mu;
use tower_lsp::{LanguageServer, lsp_types as lsp};

/// Format the test document with `code` and return the text edits.
async fn format(code: &str) -> Option<Vec<lsp::TextEdit>> {
    let service = common::service();
    common::open(&service, code).await;
    service
        .inner()
        .formatting(lsp::DocumentFormattingParams {
            text_document: lsp::TextDocumentIdentifier::new(common::url()),
            options: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .await
        .expect("No LSP error")
}

/// The code formatted like with `mucad format`.
fn formatted(code: &str) -> String {
    mu::Formatter::new()
        .format(mu::Source::from(code))
        .expect("Code without syntax errors")
        .source
        .code()
        .to_string()
}

/// The edits turn the code into the formatted code.
#[tokio::test]
async fn format_document() {
    let code =
        "sketch Disc( radius:Length ){\n__mu::geo2d::Circle(radius) ;}\n\nDisc(radius=4mm);\n";
    let edits = format(code).await.expect("Text edits");

    assert!(!edits.is_empty());
    assert_eq!(common::apply_edits(code, &edits), formatted(code));
}

/// Positions are counted in UTF-16 code units, e.g. after a `µ` in a comment.
#[tokio::test]
async fn format_document_with_unicode() {
    let code =
        "// µcad: ½ ⌀ 🚀\nx  =  1mm;\n/// Größe\nsketch S(){__mu::geo2d::Circle(radius=x);}\nS();";
    let edits = format(code).await.expect("Text edits");

    assert!(edits.iter().any(|edit| edit.range.start.line > 0));
    assert_eq!(common::apply_edits(code, &edits), formatted(code));
}

/// A formatted document needs no edits.
#[tokio::test]
async fn format_formatted_document() {
    let code = formatted("x=1mm;\n__mu::geo2d::Circle(radius=x);");
    assert_eq!(format(&code).await, Some(vec![]));
}

/// A document with syntax errors cannot be formatted.
#[tokio::test]
async fn format_document_with_syntax_error() {
    assert_eq!(format("sketch S( {").await, None);
}

/// A document which is not open cannot be formatted.
#[tokio::test]
async fn format_closed_document() {
    let service = common::service();
    let result = service
        .inner()
        .formatting(lsp::DocumentFormattingParams {
            text_document: lsp::TextDocumentIdentifier::new(common::url()),
            options: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .await;
    assert_eq!(result.expect("No LSP error"), None);
}
