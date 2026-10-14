// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI export command

use crate::{Cli, commands::RunCommand};

/// Parse and evaluate and export a µcad file.
#[derive(clap::Parser)]
pub struct Export {
    pub input: String,

    /// Output file (e.g. an SVG or STL).
    pub output: Option<std::path::PathBuf>,

    /// The resolution of this export.
    ///
    /// The resolution can changed relatively `200%` or to an absolute value `0.05mm`.
    #[arg(short, long, default_value = "0.1mm")]
    pub resolution: String,

    /// List all export target files.
    #[arg(short, long)]
    pub dry_run: bool,
}

impl RunCommand for Export {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        use microcad_driver::prelude as mu;
        use miette::IntoDiagnostic;

        let source = mu::Source::load(&self.input).into_diagnostic()?;
        let mut compiler = mu::Compiler::from_config(&cli.config);

        // Only a model without errors is exported.
        let model = match compiler.compile(source) {
            Ok((model, diag)) if !diag.has_errors() => {
                eprint!("{}", compiler.render_diag_to_string(&diag));
                model
            }
            Ok((_, diag)) | Err(diag) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));
                miette::bail!("File '{}' has errors, nothing exported.", self.input);
            }
        };

        let exporter = mu::Exporter::new().with_config(cli.config.export.clone());
        if self.dry_run {
            let path = exporter.output_path(&model, &self.input, self.output.as_deref())?;
            eprintln!("Export target: {}", path.display());
        } else {
            let path = exporter.export(&model, &self.input, self.output.as_deref())?;
            eprintln!("Exported '{}'.", path.display());
        }

        Ok(())
    }
}
