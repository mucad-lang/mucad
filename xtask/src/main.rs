// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod doctest;
mod update_copyright;

use clap::{Parser, Subcommand};
use std::error::Error;

use microcad_driver::prelude as mu;

use mu::traits::GenerateDoc;
use mu::{MUCAD_EXTENSION, MUCAD_EXTENSIONS};

use crate::doctest::TestSuite;
use crate::update_copyright::CopyrightUpdater;

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
    DocgenBuiltin {
        output_path: Option<std::path::PathBuf>,
    },

    /// Generate documentation for standard library
    DocgenStd {
        output_path: Option<std::path::PathBuf>,
    },

    /// Test an mdbook and update test results, if needed.
    Doctest { mdbook_path: std::path::PathBuf },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::UpdateCopyright => {
            CopyrightUpdater::new(".")
                .extensions(MUCAD_EXTENSIONS, "//")
                .extensions(&["rs", "slint", "wgsl"], "//")
                .extension("toml", "#")
                .exclude("target/*")
                .exclude(format!("crates/*/tests/*.{MUCAD_EXTENSION}"))
                .with_git_exclusions()
                .update()?;
            Ok(())
        }
        Commands::DocgenBuiltin { output_path } => {
            let builtin_lib = mu::BuiltinLibrary::new();
            mu::docgen::BuiltinLibraryDocGenerator::new(&builtin_lib)
                .write_doc(output_path.unwrap_or(std::path::PathBuf::from("target/docs/builtin")))
        }
        Commands::DocgenStd { output_path } => {
            let std_lib = mu::stdlib::load("crates/std/lib/std");
            mu::docgen::LibraryDocGenerator::new(&std_lib)
                .write_doc(output_path.unwrap_or(std::path::PathBuf::from("target/docs/std")))
        }
        Commands::Doctest { mdbook_path } => {
            use mu::md::WriteToFile;

            let mut mdbook = mu::md::MdBook::load(&mdbook_path)?;
            let test_suite = TestSuite::from_mdbook(&mdbook)?;

            // Only markdown files with new or changed test banners are written.
            let src_path = mdbook_path.join(mdbook.config.src_path());
            for md_file in TestSuite::set_banners(&mut mdbook) {
                eprintln!("Updated test banners in {}", md_file.display());
                mdbook.md_files[&md_file].write_to_file(src_path.join(&md_file))?;
            }

            let results = test_suite.run(&mu::DriverConfig::default());
            println!("{results}");

            if !results.is_success() {
                return Err(format!("Doctests failed: {}", results.summary()).into());
            }
            Ok(())
        }
    }
}
