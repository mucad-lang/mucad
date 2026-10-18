// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_driver::prelude::{self as mu, Artifact, traits::RenderDiag};
use miette::IntoDiagnostic;

use crate::{Cli, commands::RunCommand};

/// Defines where an artifact should be emitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitTarget {
    /// Do not emit anything.
    NoEmit,
    /// Emit to a default path, based on the input file.
    Auto,
    /// Emit to a specific path.
    Path(std::path::PathBuf),
}

impl EmitTarget {
    /// The path to emit an artifact to, or `None` if nothing shall be emitted.
    fn path(&self, default_path: impl AsRef<std::path::Path>) -> Option<std::path::PathBuf> {
        match self {
            EmitTarget::NoEmit => None,
            EmitTarget::Auto => Some(default_path.as_ref().to_path_buf()),
            EmitTarget::Path(path_buf) => Some(path_buf.clone()),
        }
    }
}

impl std::str::FromStr for EmitTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::NoEmit),
            "auto" => Ok(Self::Auto),
            _ => Ok(Self::Path(s.into())),
        }
    }
}

/// Compile a µcad file and output artifacts for debugging
#[derive(clap::Parser)]
pub struct Compile {
    /// Input µcad file.
    input: String,

    /// Emit AST (optionally specify a custom path)
    #[arg(long, num_args = 0..=1, default_value = "none", default_missing_value = "auto")]
    pub emit_ast: EmitTarget,

    /// Emit IR (optionally specify a custom path)
    #[arg(long, num_args = 0..=1, default_value = "none", default_missing_value = "auto")]
    pub emit_ir: EmitTarget,

    /// Emit Model Tree (optionally specify a custom path)
    #[arg(long, num_args = 0..=1, default_value = "none", default_missing_value = "auto")]
    pub emit_model: EmitTarget,
}

impl RunCommand<()> for Compile {
    fn run(&self, cli: &Cli) -> mu::Result {
        let source = mu::Source::load(&self.input).into_diagnostic()?;
        let mut compiler = cli.compiler();

        match compiler.compile(source.clone()) {
            Ok((model, diag)) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));

                if let Some(path) = self.emit_model.path(&self.input) {
                    model.emit(path)?;
                }
            }
            Err(err) => {
                eprintln!(
                    "⚠️ File has issues:\n{err}",
                    err = compiler.render_diag_to_string(&err)
                );

                if !matches!(self.emit_model, EmitTarget::NoEmit) {
                    eprintln!("Compilation failed, cannot emit model");
                }
            }
        }

        if let Some(path) = self.emit_ast.path(&self.input) {
            compiler.emit_ast(source.hash_id(), path)?;
        }
        if let Some(path) = self.emit_ir.path(&self.input) {
            compiler.emit_ir(source.hash_id(), path)?;
        }

        Ok(())
    }
}
