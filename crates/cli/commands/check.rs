// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_driver::prelude::traits::RenderDiag;
use miette::IntoDiagnostic;

use crate::{Cli, commands::RunCommand};

/// Check a µcad file for errors but do not render or export anything.
#[derive(clap::Parser)]
pub struct Check {
    /// Input µcad file.
    input: String,
}

impl RunCommand<()> for Check {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        use microcad_driver::prelude as mu;
        let mut compiler = cli.compiler();
        let source = mu::Source::load(&self.input).into_diagnostic()?;

        // A model compiled with recovered errors is not valid either.
        let issues = match compiler.compile(source) {
            Ok((_, issues)) if !issues.has_errors() => issues,
            Ok((_, issues)) | Err(issues) => {
                eprintln!("{}", compiler.render_diag_to_string(&issues));
                miette::bail!("File '{}' has errors.", self.input);
            }
        };

        if issues.is_empty() {
            eprintln!("✅ File '{}' is valid.", self.input);
        } else {
            eprintln!(
                "⚠️ File '{}' is valid, but has warnings:\n{}",
                self.input,
                compiler.render_diag_to_string(&issues)
            );
        }

        Ok(())
    }
}
