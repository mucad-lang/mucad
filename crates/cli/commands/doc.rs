// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI doc command

use crate::{Cli, commands::RunCommand};

/// Generate documentation from code.
#[derive(clap::Parser)]
pub struct Doc {
    /// Input file or library name.
    input: String,

    /// Output path for markdown book
    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,
}

impl RunCommand<()> for Doc {
    fn run(&self, _cli: &Cli) -> miette::Result<()> {



        todo!()
        /*use mucad_driver::prelude as mu;
        use mu::traits::*;


        let mut document = mu::Document::open(&self.input)?;
        let params = mu::DocGenParameters {
            generator_id: self.generator.clone(),
            output_path: self.output.clone(),
            resolve_parameters: match self.no_std {
                true => mu::ResolveParameters {
                    search_paths: vec![],
                },
                false => mu::ResolveParameters::default(),
            },
        };

        match document.doc_gen(params) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error generating documentation:\n{err}");
                cli.print_diagnostics(&document);
            }
        }

        Ok(())*/
    }
}
