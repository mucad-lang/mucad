// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI.

use clap::Parser;

use crate::commands::*;
use microcad_driver::prelude as mu;

/// µcad cli
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Display processing time.
    #[arg(short = 'T', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub(crate) time: bool,

    /// Load config from file.
    #[arg(short = 'C', long = "config")]
    config_path: Option<std::path::PathBuf>,

    /// Verbosity level (use -v, -vv, or -vvv)
    #[arg(short, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Subcommands.
    #[command(subcommand)]
    command: Commands,

    /// The loaded or default CLI config.
    #[clap(skip)]
    pub config: std::rc::Rc<mu::DriverConfig>,
}

impl Cli {
    /// Create a new CLI.
    pub fn new() -> miette::Result<Self> {
        let mut cli = Self::parse();

        if let Some(config_path) = &cli.config_path {
            cli.config = std::rc::Rc::new(mu::DriverConfig::load(config_path)?);
        }
        Ok(cli)
    }

    /// Run the CLI.
    pub fn run(&self) -> miette::Result<()> {
        let start = std::time::Instant::now();

        match &self.command {
            Commands::Check(check) => {
                check.run(self)?;
            }
            Commands::Compile(compile) => {
                compile.run(self)?;
            }
            Commands::Export(export) => {
                export.run(self)?;
            }
            Commands::Format(format) => {
                format.run(self)?;
            }
            Commands::Create(create) => {
                create.run(self)?;
            }
            Commands::Watch(watch) => {
                watch.run(self)?;
            }
            Commands::Completions(completions) => {
                completions.run(self)?;
            }
            Commands::Doc(doc) => {
                doc.run(self)?;
            }
        }

        if self.time {
            eprintln!(
                "Overall Time   : {}",
                Self::time_to_string(&start.elapsed())
            );
        }
        Ok(())
    }

    /// A compiler with the library settings of the config.
    pub(crate) fn compiler(&self) -> mu::Compiler {
        mu::Compiler::from_config(&self.config)
    }

    /// An exporter with the export settings of the config and a `resolution`, e.g. `0.05mm`.
    pub(crate) fn exporter(&self, resolution: &str) -> miette::Result<mu::Exporter> {
        Ok(mu::Exporter::new()
            .with_config(self.config.export.clone())
            .with_resolution(microcad_driver::resolution_from_str(resolution)?))
    }

    /// Compile `input` and print the diagnostics.
    ///
    /// Only a model without errors is returned, e.g. to export it.
    pub(crate) fn compile(
        &self,
        compiler: &mut mu::Compiler,
        input: &str,
    ) -> miette::Result<mu::ModelTree> {
        use miette::IntoDiagnostic;

        let source = mu::Source::load(input).into_diagnostic()?;

        match compiler.compile(source) {
            Ok((model, diag)) if !diag.has_errors() => {
                eprint!("{}", compiler.render_diag_to_string(&diag));
                Ok(model)
            }
            Ok((_, diag)) | Err(diag) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));
                miette::bail!("File '{input}' has errors.");
            }
        }
    }

    pub(super) fn time_to_string(duration: &std::time::Duration) -> String {
        use num_format::{Locale, ToFormattedString};
        format!(
            "{:>8}µs",
            duration.as_micros().to_formatted_string(&Locale::en)
        )
    }
}
