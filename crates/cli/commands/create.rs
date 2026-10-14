// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI create command

use crate::*;
use miette::IntoDiagnostic;

#[derive(clap::Parser)]
pub struct Create {
    path: std::path::PathBuf,
}
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "examples"]
struct Hello;

impl RunCommand for Create {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        let path = cli.config.path_with_default_ext(&self.path);

        if path.exists() {
            eprintln!("Error: File {path:?} already exists.")
        } else {
            std::fs::write(
                path.clone(),
                Hello::get("hello.mu")
                    .expect("embedded hello.mu not found")
                    .data,
            )
            .into_diagnostic()?;
            eprintln!("File {path:?} generated.")
        }

        Ok(())
    }
}
