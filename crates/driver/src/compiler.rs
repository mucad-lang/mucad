// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Compiler High-level API

pub use crate::prelude as mu;

use microcad_lang_base::{DiagRenderOptions, Diagnostics, Issue, PushIssue};
use microcad_std::{StdLib, StdLibError};

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
    /// The standard library is loaded once per compiler.
    std_loaded: bool,
}

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
            std_loaded: false,
        }
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

    /// Load the standard library once and make it an implicit dependency of every library.
    fn load_std(&mut self) -> CompileResult<()> {
        if self.no_std || self.std_loaded {
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
                self.std_loaded = true;
                Ok(((), mu::Diagnostics::default()))
            })
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
            .chain(std::iter::once(microcad_std::global_library_search_path()))
            .flat_map(|path| [path.join(info.path()), path.join(info.name.as_str())])
            .find(|path| path.join(mu::resolve::Manifest::MU_TOML).is_file());

        if let Some(path) = found {
            return Ok(path);
        }

        let path = StdLib::default_path();
        StdLib::install(&path).map_err(|StdLibError::InstallError(err)| err)?;
        Ok(path)
    }

    pub fn render_diag(
        &self,
        f: &mut dyn std::fmt::Write,
        diagnostics: &Diagnostics,
    ) -> std::fmt::Result {
        diagnostics.iter().try_for_each(|diag| {
            let hash_id = diag.src_ref.source_hash();
            match self
                .resolve_context
                .source_cache()
                .with_read(|source_cache| {
                    source_cache
                        .get(hash_id)
                        .map(|source_unit| source_unit.source())
                        .cloned()
                }) {
                Some(source) => diag.render_source(f, &source, &DiagRenderOptions::default()),
                None => Ok(()),
            }
        })
    }

    pub fn render_diag_to_string(&self, diagnostics: &Diagnostics) -> String {
        let mut buff = String::new();
        self.render_diag(&mut buff, diagnostics)
            .expect("format to string can't fail");
        buff
    }
}
