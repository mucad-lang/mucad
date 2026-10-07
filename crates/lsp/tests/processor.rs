// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test the LSP processor

use tower_lsp::{LanguageServer, lsp_types as lsp};

use microcad_lsp as mu_lsp;

use test_that::prelude::*;

/// check `lsp::InitializeParams` -> `lsp::InitializeResult`
#[tokio::test]
async fn initialize() -> std::io::Result<()> {
    let config = mu_lsp::Config::default();
    let (service, _) = mu_lsp::build_lsp_service(config).expect("No error");
    let params = lsp::InitializeParams::default();

    let backend = service.inner();
    let init_result = backend.initialize(params).await;

    assert_that!(
        init_result,
        ok(matches_pattern!(lsp::InitializeResult {
            capabilities: matches_pattern!(lsp::ServerCapabilities {
                document_formatting_provider: some(anything())
            }),
        }))
    );

    Ok(())
}
