// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

use clap::{Parser, Subcommand};
use microcad_docgen::GenerateDoc;
use microcad_lang_base::{MICROCAD_EXTENSION, MICROCAD_EXTENSIONS};

use crate::update_copyright::CopyrightUpdater;

mod update_copyright;

#[derive(Parser)]
#[command(name = "microcad-xtask")]
#[command(author, version, about = "microcad repo task executor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update copyright notices across project files
    UpdateCopyright,

    /// Generate documentation for builtins
    BuiltinDocGen { path: Option<std::path::PathBuf> },

    /// Generate documentation for standard library
    StdDocGen { path: Option<std::path::PathBuf> },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::UpdateCopyright => {
            CopyrightUpdater::new(".")
                .extensions(MICROCAD_EXTENSIONS, "//")
                .extensions(&["rs", "slint", "wgsl"], "//")
                .extension("toml", "#")
                .exclude("target/*")
                .exclude(format!("crates/*/tests/*.{MICROCAD_EXTENSION}"))
                .with_git_exclusions()
                .update()?;
            Ok(())
        }
        Commands::BuiltinDocGen { path } => {
            let builtins = microcad_builtin::BuiltinLibrary::new();
            microcad_docgen::BuiltinLibraryDocGenerator::new(&builtins)
                .write_doc(path.unwrap_or(std::path::PathBuf::from("target/docs/builtin")))
        }
        Commands::StdDocGen { .. } => {
            todo!()
        }
    }
}
