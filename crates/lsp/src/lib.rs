// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad lsp library

pub mod backend;
pub mod processor;
pub mod to_lsp;

pub use config::Config;

mod config;
mod semantic_tokens;

use backend::Service;

pub use tower_lsp as lsp;

pub use lsp::Server;

/// µcad lsp service
pub fn build_lsp_service(
    config: Config,
) -> microcad_driver::Result<(lsp::LspService<Service>, lsp::ClientSocket)> {
    log::info!("Starting LSP server");

    let processor = processor::ProcessorController::run(config.driver)?;

    Ok(
        lsp::LspService::build(|client| Service::new(client, processor, config.service))
            .custom_method("custom/activeFileChanged", Service::on_active_file_changed)
            .finish(),
    )
}
