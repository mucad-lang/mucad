// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_driver::prelude as mu;
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
    fn emit<T: mu::Artifact>(
        &self,
        artifact: &T,
        default_path: impl AsRef<std::path::Path>,
    ) -> mu::Result {
        match self {
            EmitTarget::NoEmit => Ok(()), // Don't do anything.
            EmitTarget::Auto => artifact.emit(default_path),
            EmitTarget::Path(path_buf) => artifact.emit(path_buf),
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

                self.emit_model.emit(&model, &self.input)?;
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

        compiler.with_ast(source.hash_id(), |ast| self.emit_ast.emit(ast, &self.input));
        compiler.with_ir(source.hash_id(), |ir| self.emit_ir.emit(ir, &self.input));

        Ok(())
    }
}
