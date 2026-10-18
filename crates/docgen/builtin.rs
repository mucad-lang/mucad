// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Documentation generator for µcad built-ins.

use clap::Parser;
use mucad_docgen::{self as docgen, GenerateDoc};

/// µcad built-in doc generater CLI
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct BuiltinCli {
    /// Output path (must be a directory).
    output: std::path::PathBuf,
}

fn main() -> miette::Result<()> {
    let cli = BuiltinCli::parse();
    let builtins = mucad_builtin::BuiltinLibrary::new();
    docgen::BuiltinLibraryDocGenerator::new(&builtins)
        .write_doc(cli.output)
        .map_err(|err| miette::miette!("{err}"))
}
