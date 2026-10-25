// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use miette::IntoDiagnostic;
use mucad_driver::prelude::{self as mu, Artifact, base::DisplayWithCtx, traits::RenderDiag};

use crate::{Cli, commands::RunCommand, share};

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

/// A compiler artifact to display.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayTarget {
    /// The syntax tree of a source file.
    Ast,
    /// The intermediate representation of a source file.
    Ir,
    /// The resolved symbols of a source file or library.
    #[value(alias = "lib")]
    Library,
    /// The model tree of a source file.
    Model,
    /// A link to the web viewer with the code of a source file.
    Link,
    /// The QR code of the link to the web viewer.
    #[value(alias = "qr")]
    Qrcode,
}

impl DisplayTarget {
    /// The artifacts in the order of the compiler stages, then the link with its QR code below.
    const ALL: [DisplayTarget; 6] = [
        Self::Ast,
        Self::Ir,
        Self::Library,
        Self::Model,
        Self::Link,
        Self::Qrcode,
    ];
}

impl std::fmt::Display for DisplayTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ast => "AST",
            Self::Ir => "IR",
            Self::Library => "Library",
            Self::Model => "Model",
            Self::Link => "Link",
            Self::Qrcode => "QR code",
        })
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

    /// Do not write a compiled library into `<name>.mulib` in its directory.
    #[arg(long)]
    pub no_emit_lib: bool,

    /// Display compiler artifacts, e.g. `--display ast,model`.
    /// AST, IR and model are only available for source files.
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub display: Vec<DisplayTarget>,
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

        let model = match compiler.compile(source.clone()) {
            Ok((model, diag)) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));

                if let Some(path) = self.emit_model.path(input) {
                    model.emit(path)?;
                }
                Some(model)
            }
            Err(err) => {
                eprintln!(
                    "⚠️ File has issues:\n{err}",
                    err = compiler.render_diag_to_string(&err)
                );

                if !matches!(self.emit_model, EmitTarget::NoEmit) {
                    eprintln!("Compilation failed, cannot emit model");
                }
                None
            }
        };

        if let Some(path) = self.emit_ast.path(input) {
            compiler.emit_ast(source.hash_id(), path)?;
        }
        if let Some(path) = self.emit_ir.path(input) {
            compiler.emit_ir(source.hash_id(), path)?;
        }

        self.display(|target| {
            let compiled = match target {
                DisplayTarget::Ast => compiler.with_ast(source.hash_id(), |ast| ast.to_string()),
                DisplayTarget::Ir => compiler.with_ir(source.hash_id(), |ir| ir.to_string()),
                DisplayTarget::Library => compiler.library().map(|lib| lib.to_string()),
                DisplayTarget::Model => model
                    .as_ref()
                    .map(|model| model.to_string_with_ctx(&mu::BuiltinLibrary::new())),
                // The code is shared, also with errors.
                DisplayTarget::Link => return share::link(source.code()),
                DisplayTarget::Qrcode => {
                    return share::link(source.code()).and_then(|link| share::qr_code(&link));
                }
            };
            compiled.ok_or_else(|| "compilation failed".to_string())
        });

        Ok(())
    }

    /// Print the requested artifacts in the order of the compiler stages, `artifact` returns
    /// the text of an artifact or the reason why it is not available.
    ///
    /// Several artifacts are printed with a heading each.
    fn display(&self, mut artifact: impl FnMut(DisplayTarget) -> Result<String, String>) {
        let targets: Vec<_> = DisplayTarget::ALL
            .into_iter()
            .filter(|target| self.display.contains(target))
            .collect();
        for target in &targets {
            match artifact(*target) {
                Ok(text) if targets.len() > 1 => println!("--- {target} ---\n{text}"),
                Ok(text) => println!("{text}"),
                Err(reason) => eprintln!("Cannot display {target}: {reason}"),
            }
        }
    }

    /// Compile the library in the directory `input` into `<name>.mulib` in that directory,
    /// unless `--no-emit-lib` is given.
    ///
    /// A library is not evaluated, so there is no model, and it consists of several sources.
    /// A library with errors is not written, because a `.mulib` is loaded without checking it.
    fn compile_lib(&self, compiler: &mut mu::Compiler, input: &str) -> mu::Result {
        let emits = [&self.emit_ast, &self.emit_ir, &self.emit_model];
        if emits.iter().any(|emit| **emit != EmitTarget::NoEmit) {
            eprintln!("Artifacts can only be emitted for source files, not for libraries");
        }

        match compiler.compile_lib(input) {
            Ok((_, diag)) if diag.has_errors() => Err(miette::miette!(
                "Library has issues:\n{err}",
                err = compiler.render_diag_to_string(&diag)
            )),
            Ok((lib, diag)) => {
                eprintln!("{}", compiler.render_diag_to_string(&diag));
                match lib.info() {
                    Some(info) => eprintln!("Compiled library {info}"),
                    None => eprintln!("Compiled library {input}"),
                }
                self.display(|target| match target {
                    DisplayTarget::Library => Ok(lib.to_string()),
                    DisplayTarget::Ast
                    | DisplayTarget::Ir
                    | DisplayTarget::Model
                    | DisplayTarget::Link
                    | DisplayTarget::Qrcode => Err("only available for source files".into()),
                });

                if !self.no_emit_lib {
                    let Some(name) = lib.name() else {
                        miette::bail!("The library in '{input}' has no name in its `mu.toml`");
                    };
                    let path = std::path::Path::new(input)
                        .join(format!("{name}.{}", mu::ArtifactKind::Library.ext()));
                    std::fs::write(&path, lib.to_binary()?).into_diagnostic()?;
                    eprintln!("Wrote library {}", path.display());
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
