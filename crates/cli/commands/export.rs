// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI export command

use crate::{Cli, commands::RunCommand};

/// Parse and evaluate and export a µcad file.
#[derive(clap::Parser)]
pub struct Export {
    pub input: String,

    /// Output file (e.g. an SVG or STL) for the whole model.
    ///
    /// Without output, the model is exported into the files of its export attributes
    /// (e.g. `#[export = "logo.svg"]`) or, without any, next to the input.
    pub output: Option<std::path::PathBuf>,

    /// The resolution of this export.
    ///
    /// Either an absolute length, e.g. `0.05mm`, or relative to the default resolution of
    /// 0.25mm, e.g. `200%` for a twice as fine resolution.
    #[arg(short, long, default_value = "100%")]
    pub resolution: String,

    /// List all export target files.
    #[arg(short, long)]
    pub dry_run: bool,
}

impl RunCommand for Export {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        let mut compiler = cli.compiler();
        let model = cli.compile(&mut compiler, &self.input)?;

        let exporter = cli.exporter(&self.resolution)?;

        // Without output, the export attributes of the source decide the files, if it has some.
        let files = match &self.output {
            Some(_) => Vec::new(),
            None => exporter.export_files(&model, &self.input)?,
        };
        if !files.is_empty() {
            for file in &files {
                if self.dry_run {
                    eprintln!("Export target: {}", file.path.display());
                } else {
                    let path = exporter.export_file(file)?;
                    eprintln!("Exported '{}'.", path.display());
                }
            }
            return Ok(());
        }

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
