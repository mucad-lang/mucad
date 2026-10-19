// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test code completion with the LSP service.

mod common;

use tower_lsp::{LanguageServer, lsp_types as lsp};

/// A document with a sketch, a module and a string with characters outside of ASCII.
const CODE: &str = r#"/// A disc.
sketch Disc() {}

mod geo {
    pub sketch Square() {}
    sketch Hidden() {}
}

s = "µ🚀";
"#;

/// The completion items at `|` in `code`, after the document has been opened with [`CODE`]
/// and changed into `code`, like while typing.
async fn complete(code: &str) -> Vec<lsp::CompletionItem> {
    let service = common::service();
    common::open(&service, CODE).await;

    let offset = code.find('|').expect("A position");
    let code = code.replacen('|', "", 1);
    service
        .inner()
        .did_change(lsp::DidChangeTextDocumentParams {
            text_document: lsp::VersionedTextDocumentIdentifier::new(common::url(), 2),
            content_changes: vec![lsp::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: code.clone(),
            }],
        })
        .await;

    let response = service
        .inner()
        .completion(lsp::CompletionParams {
            text_document_position: lsp::TextDocumentPositionParams::new(
                lsp::TextDocumentIdentifier::new(common::url()),
                position(&code, offset),
            ),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        })
        .await
        .expect("No LSP error");
    match response {
        Some(lsp::CompletionResponse::Array(items)) => items,
        response => panic!("Completion items: {response:?}"),
    }
}

/// The LSP position (UTF-16 columns) of a byte offset.
fn position(code: &str, offset: usize) -> lsp::Position {
    let before = &code[..offset];
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    lsp::Position::new(
        before.matches('\n').count() as u32,
        before[line_start..].encode_utf16().count() as u32,
    )
}

fn labels(items: &[lsp::CompletionItem]) -> Vec<&str> {
    items.iter().map(|item| item.label.as_str()).collect()
}

/// Names of the document with their kind and documentation, e.g. while typing `Di`.
///
/// The document is the file module `main` (`main.mu`), whose symbols can also be used by path.
#[tokio::test]
async fn complete_names() {
    let items = complete(&format!("{CODE}Di|")).await;
    assert_eq!(labels(&items), ["Disc", "geo", "main"]);

    let disc = &items[0];
    assert_eq!(disc.kind, Some(lsp::CompletionItemKind::CLASS));
    assert_eq!(disc.detail.as_deref(), Some("sketch"));
    assert_eq!(
        disc.documentation,
        Some(lsp::Documentation::MarkupContent(lsp::MarkupContent {
            kind: lsp::MarkupKind::Markdown,
            value: "A disc.".into(),
        }))
    );
    assert_eq!(items[1].kind, Some(lsp::CompletionItemKind::MODULE));
}

/// The public symbols of a module after `::`, behind characters outside of ASCII in the line.
#[tokio::test]
async fn complete_module() {
    let code = CODE.replace(r#"s = "µ🚀";"#, r#"s = "µ🚀"; geo::|"#);
    assert_eq!(labels(&complete(&code).await), ["Square"]);
}

/// Built-ins and numbers are not completed.
#[tokio::test]
async fn complete_nothing() {
    assert!(complete(&format!("{CODE}__mu::geo2d::|")).await.is_empty());
    assert!(complete(&format!("{CODE}x = 12|")).await.is_empty());
}

/// The server completes paths after `::`.
#[tokio::test]
async fn completion_capability() {
    let result = common::service()
        .inner()
        .initialize(lsp::InitializeParams::default())
        .await
        .expect("No LSP error");
    let trigger_characters = result
        .capabilities
        .completion_provider
        .and_then(|options| options.trigger_characters);
    assert_eq!(trigger_characters, Some(vec![":".into()]));
}
