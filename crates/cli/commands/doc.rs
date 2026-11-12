// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI doc command

use mucad_driver::prelude::base::RenderDiag;

use crate::{Cli, commands::RunCommand};

/// Generate documentation from µcad source.
#[derive(clap::Parser)]
pub struct Doc {
    /// Input file or library name.
    input: String,

    /// Output path for markdown book
    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,
}

impl RunCommand<()> for Doc {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        use mu::docgen::GenerateDoc;
        use mucad_driver::prelude as mu;

        let mut compiler = cli.compiler();

        match compiler.compile_lib(&self.input) {
            Ok((lib, diag)) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));

                mu::docgen::LibraryDocGenerator::new(&lib)
                    .write_doc(
                        self.output
                            .clone()
                            .unwrap_or(std::path::PathBuf::from(lib.name().unwrap())),
                    )
                    .map_err(|err| miette::miette!("{err}"))?;

                Ok(())
            }
            Err(diag) => Err(miette::miette!(
                "Library has issues:\n{err}",
                err = compiler.render_diag_to_string(&diag)
            )),
        }
    }
}
