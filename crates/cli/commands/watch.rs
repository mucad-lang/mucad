// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI watch command

use crate::*;

#[derive(clap::Parser)]
pub struct Watch {
    pub input: String,

    /// Output file (e.g. an SVG or STL).
    pub output: Option<std::path::PathBuf>,

    /// The resolution of this export.
    ///
    /// Either an absolute length, e.g. `0.05mm`, or relative to the default resolution of
    /// 0.25mm, e.g. `200%` for a twice as fine resolution.
    #[arg(short, long, default_value = "100%")]
    pub resolution: String,
}

/// Run this command for a CLI.
impl RunCommand for Watch {
    fn run(&self, cli: &Cli) -> miette::Result<()> {
        use microcad_driver::prelude as mu;

        let mut watcher = mu::Watcher::new()?;
        // The compiler keeps the standard library and unchanged sources between compilations.
        let mut compiler = cli.compiler();
        let exporter = cli.exporter(&self.resolution)?;

        // Recompile and export whenever a source file changes.
        loop {
            match cli
                .compile(&mut compiler, &self.input)
                .and_then(|model| exporter.export(&model, &self.input, self.output.as_deref()))
            {
                Ok(path) => eprintln!("Exported '{}'.", path.display()),
                Err(err) => eprintln!("{err:?}"),
            }

            // Watch the input and the file modules of the most recent compilation.
            let input = std::path::PathBuf::from(&self.input);
            watcher.update(std::iter::once(input).chain(compiler.source_files()))?;

            // Wait until anything relevant happens.
            watcher.wait()?;
        }
    }
}
