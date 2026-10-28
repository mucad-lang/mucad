// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod change_version;
mod check_readme;
mod doctest;
mod update_copyright;

use clap::{Parser, Subcommand};
use std::error::Error;

use mucad_driver::prelude as mu;

use mu::traits::GenerateDoc;
use mu::{MUCAD_EXTENSION, MUCAD_EXTENSIONS};

use crate::doctest::TestSuite;
use crate::update_copyright::CopyrightUpdater;

#[derive(Parser)]
#[command(name = "mucad-xtask")]
#[command(author, version, about = "mucad repo task executor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update copyright notices across project files
    UpdateCopyright,

    /// Change the version of the whole workspace, e.g. `0.6.1`, and update `Cargo.lock`
    ChangeVersion {
        /// The new version, e.g. `0.6.1` or `0.7.0-rc.1`
        version: String,
        /// Only list the files to change
        #[arg(long)]
        dry_run: bool,
    },

    /// Run the shell commands of README.md files one by one, to check them manually
    CheckReadme {
        /// README.md files or directories to search, by default the whole repository
        paths: Vec<std::path::PathBuf>,
        /// Only list the commands
        #[arg(long)]
        list: bool,
    },

    /// Generate documentation for builtins
    DocgenBuiltin {
        output_path: Option<std::path::PathBuf>,
    },

    /// Generate documentation for standard library
    DocgenStd {
        output_path: Option<std::path::PathBuf>,
    },

    /// Test an mdbook and update test results, if needed.
    Doctest {
        /// Do not write any output files (no export and no logs)
        #[arg(long)]
        no_output: bool,
        /// The path to the md book
        mdbook_path: std::path::PathBuf,
        /// An optional wildcard string to run a test subset.
        wildcard: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::UpdateCopyright => {
            // Assign AGPLv3.0 license by default.
            CopyrightUpdater::new(".")
                .extensions(&["rs", "slint", "wgsl"], "//")
                .exclude("target/*")
                .with_git_exclusions()
                .update()?;

            // Assign MIT license to .mu files.
            CopyrightUpdater::new(".")
                .extensions(MUCAD_EXTENSIONS, "//")
                .license("MIT")
                .exclude("target/*")
                .exclude(format!("crates/*/tests/*.{MUCAD_EXTENSION}"))
                .with_git_exclusions()
                .update()?;

            Ok(())
        }
        Commands::ChangeVersion { version, dry_run } => {
            change_version::change_version(&version, dry_run)
        }
        Commands::CheckReadme { paths, list } => {
            let paths = if paths.is_empty() {
                vec![std::path::PathBuf::from(".")]
            } else {
                paths
            };
            check_readme::check_readme(&paths, list)
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
        Commands::Doctest {
            mdbook_path,
            no_output,
            wildcard,
        } => {
            let mdbook = mu::md::MdBook::load(&mdbook_path)?;

            let mut test_suite = TestSuite::from_mdbook(&mdbook)?;
            if let Some(wildcard) = wildcard
                && let Ok(regex) = regex::Regex::new(&wildcard)
            {
                test_suite.filter_test_cases(regex);
            }

            let results = test_suite.run(no_output, &mu::DriverConfig::default());
            println!("{results}");

            if !results.is_success() {
                return Err(format!("Doctests failed: {}", results.summary()).into());
            }
            Ok(())
        }
    }
}
