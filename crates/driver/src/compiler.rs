// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Compiler High-level API

pub use crate::prelude as mu;

use std::sync::Arc;

use mucad_lang_base::{DiagRenderOptions, Diagnostics, Issue, PushIssue, RenderDiag};
use mucad_std::{StdLib, StdLibError};

/// Result of the compiler: the output bundled with the diagnostics of all stages run so far.
///
/// - `Ok`: The stage produced an output. The diagnostics may contain errors the stage recovered from.
/// - `Err`: The stage produced no output. The diagnostics contain at least one error.
///
/// A single language stage returns an [`mu::IssueResult`] with typed issues instead,
/// which is converted with [`IssueResultExt::into_compile_result`].
pub type CompileResult<T> = Result<(T, mu::Diagnostics), mu::Diagnostics>;

/// Convert the typed issues of a language stage into diagnostics.
pub trait IssueResultExt<T> {
    fn into_compile_result(self) -> CompileResult<T>;
}

impl<T, I: Issue> IssueResultExt<T> for mu::IssueResult<T, I> {
    fn into_compile_result(self) -> CompileResult<T> {
        self.map(|(value, issues)| (value, issues.into_diagnostics()))
            .map_err(|issues| issues.into_diagnostics())
    }
}

/// Chain compiler stages.
pub trait CompileResultExt<T> {
    /// Run the next stage on the output and merge the diagnostics of both stages.
    ///
    /// If this stage failed, the next stage is not run and the diagnostics are passed on.
    fn then<U>(self, next: impl FnOnce(T) -> CompileResult<U>) -> CompileResult<U>;
}

impl<T> CompileResultExt<T> for CompileResult<T> {
    fn then<U>(self, next: impl FnOnce(T) -> CompileResult<U>) -> CompileResult<U> {
        let (value, mut diags) = self?;
        match next(value) {
            Ok((value, next_diags)) => {
                diags.append(next_diags);
                Ok((value, diags))
            }
            Err(next_diags) => {
                diags.append(next_diags);
                Err(diags)
            }
        }
    }
}

pub struct Compiler {
    resolve_context: mu::ResolveContext,
    /// Do not load the standard library.
    no_std: bool,
    /// Paths to search for the standard library.
    search_paths: Vec<std::path::PathBuf>,
    /// The standard library, which is loaded once per compiler.
    std: Option<mu::base::LibraryId>,
}

/// Symbols of the standard library which can be used by name, e.g. `Circle` instead of `std::geo2d::Circle`.
const PRELUDE: &[&str] = &[
    "geo2d::Circle",
    "geo2d::Rect",
    "ops::extrude",
    "ops::revolve",
    "ops::translate",
    "ops::rotate",
    "ops::scale",
    "ops::orient",
    "ops::hull",
    "print",
    "assert",
    "assert_eq",
];

impl Compiler {
    /// A compiler with the default configuration, which loads the standard library.
    pub fn new() -> Compiler {
        Self::from_config(&mu::DriverConfig::default())
    }

    /// A compiler with the library settings of a driver config.
    pub fn from_config(config: &mu::DriverConfig) -> Compiler {
        Self {
            resolve_context: mu::ResolveContext::new(),
            no_std: config.no_std,
            search_paths: config.search_paths.clone(),
            std: None,
        }
    }

    /// Load sources and libraries from `file_system`, e.g. a virtual one in a browser.
    pub fn with_file_system(mut self, file_system: Arc<dyn mu::base::FileSystem>) -> Self {
        self.resolve_context = mu::ResolveContext::builder()
            .with_file_system(file_system)
            .build();
        self
    }
}

impl Compiler {
    /// Compile a source into a model tree.
    ///
    /// Parsing and lowering happen while resolving, because resolving loads further source files.
    pub fn compile(&mut self, source: impl Into<mu::Source>) -> CompileResult<mu::ModelTree> {
        self.load_std()
            .then(|()| {
                mu::resolve::resolve_source(source, &mut self.resolve_context).into_compile_result()
            })
            .then(|lib| {
                let lib_cache = self.resolve_context.lib_cache();
                let lib_id = lib_cache.with_write(|lib_cache| lib_cache.insert_root(lib));
                let eval_context = mu::EvalContext::builder()
                    .with_lib_cache(lib_cache)
                    .with_lib_id(lib_id)
                    .build();
                mu::eval::eval(eval_context).into_compile_result()
            })
    }

    /// All source files the most recent compilation depends on, e.g. to watch them for changes.
    ///
    /// These are the compiled source, its file modules and the source files of the libraries
    /// it uses, e.g. the standard library.
    /// If the last compilation failed while loading, the files of the compilation before are returned.
    pub fn source_files(&self) -> Vec<std::path::PathBuf> {
        self.resolve_context
            .lib_cache()
            .with_read(|lib_cache| lib_cache.source_files(&mu::base::LibraryId::ROOT))
    }

    /// The symbols to complete the path in front of the byte `offset` in `source`, e.g. the
    /// symbols of `std::geo2d` for `std::geo2d::Ci`, or the visible names for `Ci`.
    ///
    /// The symbols are those of the most recent compilation, which may have been of another
    /// version of `source`, see [`mu::ResolveContext::completions`].
    /// Numbers (e.g. `12`) are not completed.
    pub fn completions(&self, source: &mu::Source, offset: usize) -> Vec<mu::resolve::Completion> {
        match qualifier_in_front_of(source.code(), offset) {
            Some(qualifier) => self.resolve_context.completions(source, offset, &qualifier),
            None => Vec::new(),
        }
    }

    /// Execute a closure with a reference to the AST, if present.
    pub fn with_ast<F, R>(&self, id: mu::HashId, f: F) -> Option<R>
    where
        F: FnOnce(&mu::Ast) -> R,
    {
        self.resolve_context.with_ast(id, f)
    }

    /// Execute a closure with a reference to the IR, if present.
    pub fn with_ir<F, R>(&self, id: mu::HashId, f: F) -> Option<R>
    where
        F: FnOnce(&mu::Ir) -> R,
    {
        self.resolve_context.with_ir(id, f)
    }

    /// Write the AST of the source `id` into `path` with the extension `ast`, e.g. `circle.mu.ast`.
    ///
    /// Returns the path of the written file.
    pub fn emit_ast(
        &self,
        id: mu::HashId,
        path: impl AsRef<std::path::Path>,
    ) -> mu::Result<std::path::PathBuf> {
        self.resolve_context.emit_ast(id, path)
    }

    /// Write the IR of the source `id` into `path` with the extension `ir`, e.g. `circle.mu.ir`.
    ///
    /// Returns the path of the written file.
    pub fn emit_ir(
        &self,
        id: mu::HashId,
        path: impl AsRef<std::path::Path>,
    ) -> mu::Result<std::path::PathBuf> {
        self.resolve_context.emit_ir(id, path)
    }

    /// Load the standard library once and make it an implicit dependency of every library.
    fn load_std(&mut self) -> CompileResult<()> {
        if self.no_std || self.std.is_some() {
            return Ok(((), mu::Diagnostics::default()));
        }

        let path = match self.std_path() {
            Ok(path) => path,
            Err(err) => {
                let mut issues = mu::base::IssueList::<mu::resolve::ResolveIssue>::new();
                issues.push_err(err);
                return Err(issues.into_diagnostics());
            }
        };

        self.resolve_context
            .load_external_library(&path)
            .into_compile_result()
            .then(|lib_id| {
                self.resolve_context
                    .add_implicit_dependency(StdLib::info().name, lib_id);
                self.std = Some(lib_id);
                let prelude = self.prelude();
                self.resolve_context.set_prelude(prelude);
                Ok(((), mu::Diagnostics::default()))
            })
    }

    /// The prelude with the [`PRELUDE`] symbols of the standard library.
    ///
    /// Without a loaded standard library the prelude is empty.
    fn prelude(&self) -> mu::resolve::Prelude {
        let Some(lib_id) = self.std else {
            return mu::resolve::Prelude::new(mu::base::LibraryId::ROOT);
        };
        let mut prelude = mu::resolve::Prelude::new(lib_id);
        let Some(lib) = self
            .resolve_context
            .lib_cache()
            .with_read(|lib_cache| lib_cache.get(&lib_id))
        else {
            return prelude;
        };

        let std_name = StdLib::info().name;
        for path in PRELUDE {
            let Ok(full_path) =
                format!("::{std_name}::{path}").parse::<mu::lower::ir::UnresolvedPath>();
            match (lib.look_up(lib.root, &full_path), full_path.parts.last()) {
                (Some(node_id), Some(name)) => {
                    prelude
                        .symbols
                        .insert(name.clone(), lib.symbol_id(node_id, Some(lib_id)));
                }
                _ => log::warn!("Prelude symbol `{full_path}` not found in the standard library"),
            }
        }
        prelude
    }

    /// Find the standard library in the search paths, or install it into its default path.
    ///
    /// A search path contains either an installed `std-<version>` folder
    /// or a `std` folder (e.g. `crates/std/lib` in the µcad repository).
    fn std_path(&self) -> Result<std::path::PathBuf, mu::resolve::ResolveError> {
        let info = StdLib::info();
        let found = self
            .search_paths
            .iter()
            .cloned()
            .chain(mucad_std::global_library_search_path())
            .flat_map(|path| [path.join(info.path()), path.join(info.name.as_str())])
            .find(|path| {
                let file_system = self.resolve_context.file_system();
                file_system.is_file(&path.join(mu::resolve::Manifest::MU_TOML))
            });

        if let Some(path) = found {
            return Ok(path);
        }

        let path = StdLib::default_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Standard library not found")
        })?;
        StdLib::install(&path).map_err(|StdLibError::InstallError(err)| err)?;
        Ok(path)
    }
}

/// Diagnostics are rendered with the source code from the compiler's source cache.
impl RenderDiag for Compiler {
    fn render_diag_with(
        &self,
        f: &mut dyn std::fmt::Write,
        diagnostics: &Diagnostics,
        options: &DiagRenderOptions,
    ) -> std::fmt::Result {
        self.resolve_context
            .source_cache()
            .with_read(|source_cache| {
                diagnostics
                    .iter()
                    .try_for_each(|diag| diag.render(f, source_cache, options))
            })
    }
}

/// The qualifier of the path in front of the byte `offset` in `code`, e.g. `std::geo2d` for
/// `std::geo2d::Ci`, `::` for `::Ci` or no parts for `Ci`.
///
/// Returns `None` in front of a number or inside of a character.
fn qualifier_in_front_of(code: &str, offset: usize) -> Option<mu::lower::ir::UnresolvedPath> {
    let before = code.get(..offset)?;
    let is_identifier_char = |ch: char| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'';

    // Identifiers separated by `::`, e.g. `std::geo2d::Ci`.
    let mut start = offset;
    loop {
        start = before[..start].trim_end_matches(is_identifier_char).len();
        match before[..start].strip_suffix("::") {
            Some(rest) => start = rest.len(),
            None => break,
        }
    }

    let path = &before[start..];
    let (qualifier, name) = match path.rsplit_once("::") {
        // `::Ci` is an absolute path in the library root.
        Some(("", name)) => ("::", name),
        Some(split) => split,
        None => ("", path),
    };
    if name.starts_with(|ch: char| ch.is_ascii_digit()) {
        return None;
    }
    qualifier.parse().ok()
}
