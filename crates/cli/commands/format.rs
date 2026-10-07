// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Cli, commands::RunCommand};

/// Format a µcad file.
#[derive(clap::Parser)]

pub struct Format {
    /// Input µcad file if not set, we read from stdin.
    pub input: Option<String>,
}

impl RunCommand<()> for Format {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        use microcad_driver::prelude as mu;
        use mu::traits::Format;
        let params = mu::FormatParameters::default();

        match &self.input {
            Some(input) => {
                let mut document = mu::Document::open(input)?;

                /*
                match document.parse().and(document.format(&params)) {
                    Ok(true) => {
                        document.sync()?;
                        eprintln!("Formatted document.");
                    }
                    Ok(false) => {
                        eprintln!("Document has been already formatted.");
                    }
                    Err(_) => {
                        cli.print_diagnostics(&document);
                    }
                }
                */
                todo!();

                Ok(())
            }
            None => match mu::document::Stdin.format(&params).map(|_| ()) {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!("{err}");
                    Ok(())
                }
            },
        }
    }
}
