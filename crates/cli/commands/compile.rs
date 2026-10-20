// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use miette::IntoDiagnostic;
use mucad_driver::prelude::{self as mu, Artifact, traits::RenderDiag};

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

/// Compile a µcad file or library and output artifacts for debugging
#[derive(clap::Parser)]
pub struct Compile {
    /// Input µcad file or library directory (with `mu.toml` and `lib.mu`).
    /// Without input, the current directory is compiled as library.
    input: Option<String>,

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
        let mut compiler = cli.compiler();
        let input = self.input.as_deref().unwrap_or(".");

        if compiler.is_library_dir(input) {
            self.compile_lib(&mut compiler, input)
        } else if self.input.is_none() {
            Err(miette::miette!(
                "The current directory is no library (with `mu.toml` and `lib.mu`), \
                 give a µcad file or library directory"
            ))
        } else {
            self.compile_source(&mut compiler, input)
        }
    }
}

impl Compile {
    /// Compile the source file `input` and emit its artifacts.
    fn compile_source(&self, compiler: &mut mu::Compiler, input: &str) -> mu::Result {
        let source = mu::Source::load(input).into_diagnostic()?;

        match compiler.compile(source.clone()) {
            Ok((model, diag)) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));

                if let Some(path) = self.emit_model.path(input) {
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

        if let Some(path) = self.emit_ast.path(input) {
            compiler.emit_ast(source.hash_id(), path)?;
        }
        if let Some(path) = self.emit_ir.path(input) {
            compiler.emit_ir(source.hash_id(), path)?;
        }

        Ok(())
    }

    /// Compile the library in the directory `input`.
    ///
    /// A library is not evaluated, so there is no model, and it consists of several sources.
    fn compile_lib(&self, compiler: &mut mu::Compiler, input: &str) -> mu::Result {
        let emits = [&self.emit_ast, &self.emit_ir, &self.emit_model];
        if emits.iter().any(|emit| **emit != EmitTarget::NoEmit) {
            eprintln!("Artifacts can only be emitted for source files, not for libraries");
        }

        match compiler.compile_lib(input) {
            Ok((lib, diag)) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));
                match lib.info() {
                    Some(info) => eprintln!("Compiled library {info}"),
                    None => eprintln!("Compiled library {input}"),
                }
                Ok(())
            }
            Err(err) => Err(miette::miette!(
                "Library has issues:\n{err}",
                err = compiler.render_diag_to_string(&err)
            )),
        }
    }
}
