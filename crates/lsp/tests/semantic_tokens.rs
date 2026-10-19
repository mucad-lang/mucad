// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test semantic tokens with the LSP service.

mod common;

use tower_lsp::{LanguageServer, lsp_types as lsp};

/// The semantic tokens of the test document with `code` at their absolute positions.
async fn semantic_tokens(code: &str) -> Vec<(lsp::Position, u32)> {
    let service = common::service();
    common::open(&service, code).await;
    let result = service
        .inner()
        .semantic_tokens_full(lsp::SemanticTokensParams {
            text_document: lsp::TextDocumentIdentifier::new(common::url()),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await
        .expect("No LSP error");
    let Some(lsp::SemanticTokensResult::Tokens(tokens)) = result else {
        panic!("Semantic tokens: {result:?}");
    };

    let mut position = lsp::Position::default();
    tokens
        .data
        .iter()
        .map(|token| {
            position = match token.delta_line {
                0 => lsp::Position::new(position.line, position.character + token.delta_start),
                delta_line => lsp::Position::new(position.line + delta_line, token.delta_start),
            };
            (position, token.length)
        })
        .collect()
}

/// Tokens are encoded in the order of their positions, also after unicode characters.
#[tokio::test]
async fn tokens_in_order() {
    let tokens = semantic_tokens(
        "/// µ doc\nsketch Disc(radius: Length) {\n    __mu::geo2d::Circle(radius);\n}",
    )
    .await;

    // The doc comment and the `sketch` keyword.
    assert_eq!(tokens[0], (lsp::Position::new(0, 0), 9));
    assert!(
        tokens.contains(&(lsp::Position::new(1, 0), 6)),
        "{tokens:?}"
    );
    assert!(tokens.is_sorted(), "{tokens:?}");
}

/// A document with syntax errors has no semantic tokens.
#[tokio::test]
async fn no_tokens_with_syntax_error() {
    let service = common::service();
    common::open(&service, "sketch S( {").await;
    let result = service
        .inner()
        .semantic_tokens_full(lsp::SemanticTokensParams {
            text_document: lsp::TextDocumentIdentifier::new(common::url()),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await;
    assert_eq!(result.expect("No LSP error"), None);
}
