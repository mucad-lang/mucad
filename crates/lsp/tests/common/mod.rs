// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Helpers to test the LSP service like an editor.

// Each test crate uses only some of the helpers.
#![allow(dead_code)]

use mucad_driver::prelude as mu;
use mucad_lsp as mu_lsp;
use tower_lsp::{LanguageServer, LspService, lsp_types as lsp};

/// An LSP service without standard library, independent of any installation.
pub fn service() -> LspService<mu_lsp::backend::Service> {
    let config = mu_lsp::Config {
        driver: mu::DriverConfig {
            no_std: true,
            ..Default::default()
        },
        ..Default::default()
    };
    mu_lsp::build_lsp_service(config).expect("A service").0
}

/// The URL of a test document.
pub fn url() -> lsp::Url {
    lsp::Url::parse("file:///test/main.mu").expect("A URL")
}

/// Open the test document with `code`.
pub async fn open(service: &LspService<mu_lsp::backend::Service>, code: &str) {
    service
        .inner()
        .did_open(lsp::DidOpenTextDocumentParams {
            text_document: lsp::TextDocumentItem::new(url(), "mucad".into(), 1, code.into()),
        })
        .await;
}

/// Apply text edits to `text` like an editor: all edits refer to the original text.
pub fn apply_edits(text: &str, edits: &[lsp::TextEdit]) -> String {
    let mut ranges: Vec<_> = edits
        .iter()
        .map(|edit| {
            (
                offset(text, edit.range.start)..offset(text, edit.range.end),
                &edit.new_text,
            )
        })
        .collect();
    // Applied from the end, so the offsets of the edits before stay valid.
    ranges.sort_by_key(|(range, _)| range.start);
    let mut text = text.to_string();
    for (range, new_text) in ranges.into_iter().rev() {
        text.replace_range(range, new_text);
    }
    text
}

/// The byte offset of an LSP position (zero-based line, UTF-16 column) in `text`.
fn offset(text: &str, position: lsp::Position) -> usize {
    let line_start: usize = text
        .split_inclusive('\n')
        .take(position.line as usize)
        .map(str::len)
        .sum();
    let mut units = 0;
    for (index, ch) in text[line_start..].char_indices() {
        if units >= position.character || ch == '\n' {
            return line_start + index;
        }
        units += ch.len_utf16() as u32;
    }
    text.len()
}
