// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use miette::IntoDiagnostic;

use crate::{Cli, commands::RunCommand};

/// Format a µcad file.
#[derive(clap::Parser)]

pub struct Format {
    /// Input µcad file. If not set, we read from stdin and write to stdout.
    pub input: Option<String>,
}

impl RunCommand<()> for Format {
    fn run(&self, _cli: &Cli) -> miette::Result<()> {
        use mucad_driver::prelude as mu;
        let formatter = mu::Formatter::new();

        let source = match &self.input {
            Some(input) => mu::Source::load(&input).into_diagnostic()?,
            None => mu::Source::stdin().into_diagnostic()?,
        };

        match formatter.format(source.clone()) {
            Ok(mu::Formatted {
                source, formatted, ..
            }) => {
                if self.input.is_none() {
                    // A source from stdin has no file to sync to.
                    print!("{}", source.code());
                } else if formatted {
                    source.sync().into_diagnostic()?;
                    eprintln!("Formatted document.");
                } else {
                    eprintln!("Document has been already formatted.");
                }
            }
            Err(err) => {
                eprintln!(
                    "Parse error:\n{}",
                    err.render_to_string(&source, &Default::default())
                        .into_diagnostic()?
                );
            }
        }

        Ok(())
    }
}
