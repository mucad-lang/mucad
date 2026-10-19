// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

/// The name of the µcad language server binary, e.g. installed with `cargo install`.
const LSP_BINARY_NAME: &str = "mucad-lsp";

struct MucadExtension;

impl zed::Extension for MucadExtension {
    fn new() -> Self {
        Self
    }

    /// The language server command: the binary from the settings
    /// (`lsp.mucad-lsp.binary.path`) or from the `PATH`, which is started with `--stdio`.
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let command = binary
            .as_ref()
            .and_then(|binary| binary.path.clone())
            .or_else(|| worktree.which(LSP_BINARY_NAME))
            .ok_or_else(|| {
                format!(
                    "Cannot find `{LSP_BINARY_NAME}`: install it with \
                     `cargo install --path crates/lsp` or set `lsp.{LSP_BINARY_NAME}.binary.path`"
                )
            })?;

        let mut env = worktree.shell_env();
        env.extend(
            binary
                .as_ref()
                .and_then(|binary| binary.env.clone())
                .unwrap_or_default(),
        );

        Ok(zed::Command {
            command,
            args: binary
                .and_then(|binary| binary.arguments)
                .unwrap_or_else(|| vec!["--stdio".into()]),
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

zed::register_extension!(MucadExtension);
