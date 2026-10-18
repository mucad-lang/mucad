// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cache;
mod lib_ctx;
mod loader;
mod prelude;

pub use cache::{LibraryCache, SourceCache};
pub use lib_ctx::ResolveLibraryContext;
use mucad_lang_parse::Ast;
pub use prelude::Prelude;
use std::collections::BTreeMap;

use mucad_lang_base::{
    Artifact, HashId, IssueList, IssueResult, LibraryId, Name, PushIssue, SharedMut,
};
use mucad_lang_lower::Ir;

use crate::{Library, ResolveErrorKind, ResolveIssue};

#[derive(Debug, Default)]
pub struct ResolveContext {
    _lib_search_paths: Vec<std::path::PathBuf>,
    lib_cache: SharedMut<LibraryCache>,
    src_cache: SharedMut<SourceCache>,
    /// Libraries in the cache that every resolved library depends on, e.g. `std`.
    implicit_dependencies: BTreeMap<Name, LibraryId>,
    /// Paths of the external libraries being loaded, to detect cyclic dependencies.
    loading: Vec<std::path::PathBuf>,
    /// Symbols which can be used by name without `use`, e.g. of `std`.
    prelude: Option<Prelude>,
}

/// Builder methods.
impl ResolveContext {
    /// Create a `ResolveContext` with default search paths and fresh caches.
    pub fn new() -> Self {
        ResolveContextBuilder::new().build()
    }

    /// Create a builder for customized resolver setup.
    pub fn builder() -> ResolveContextBuilder {
        ResolveContextBuilder::new()
    }
}

/// Accessors
impl ResolveContext {
    /// Execute a closure with a reference to the AST, if present.
    pub fn with_ast<F, R>(&self, id: HashId, f: F) -> Option<R>
    where
        F: FnOnce(&Ast) -> R,
    {
        self.src_cache
            .with_read(|src_cache| src_cache.get(id).and_then(|unit| unit.ast().map(f)))
    }

    /// Execute a closure with a reference to the IR, if present.
    pub fn with_ir<F, R>(&self, id: HashId, f: F) -> Option<R>
    where
        F: FnOnce(&Ir) -> R,
    {
        self.src_cache
            .with_read(|src_cache| src_cache.get(id).and_then(|unit| unit.ir().map(f)))
    }

    /// Get intermediate representation by hash id.
    pub fn get_ir(&self, id: HashId) -> Option<Ir> {
        self.src_cache.with_read(|src_cache| {
            src_cache
                .get(id)
                .and_then(|source_unit| source_unit.ir().cloned())
        })
    }

    /// Write the AST of the source `id` into `path` with the extension `ast`.
    ///
    /// Returns the path of the written file.
    pub fn emit_ast(
        &self,
        id: HashId,
        path: impl AsRef<std::path::Path>,
    ) -> miette::Result<std::path::PathBuf> {
        self.with_ast(id, |ast| ast.emit(path))
            .transpose()?
            .ok_or_else(|| miette::miette!("No AST to emit"))
    }

    /// Write the IR of the source `id` into `path` with the extension `ir`.
    ///
    /// Returns the path of the written file.
    pub fn emit_ir(
        &self,
        id: HashId,
        path: impl AsRef<std::path::Path>,
    ) -> miette::Result<std::path::PathBuf> {
        self.with_ir(id, |ir| ir.emit(path))
            .transpose()?
            .ok_or_else(|| miette::miette!("No IR to emit"))
    }

    /// Source cache
    pub fn source_cache(&self) -> SharedMut<SourceCache> {
        self.src_cache.clone()
    }

    /// Library cache
    pub fn lib_cache(&self) -> SharedMut<LibraryCache> {
        self.lib_cache.clone()
    }

    /// Libraries every resolved library depends on, unless its manifest says `no_std`.
    pub fn implicit_dependencies(&self) -> &BTreeMap<Name, LibraryId> {
        &self.implicit_dependencies
    }
}

/// Library loading
impl ResolveContext {
    /// Load and resolve an external library (a dependency) from a directory with a `mu.toml` file
    /// and insert it into the library cache.
    ///
    /// A library, which depends on itself through its dependencies, is an error.
    pub fn load_external_library(
        &mut self,
        lib_path: impl AsRef<std::path::Path>,
    ) -> IssueResult<LibraryId, ResolveIssue> {
        let lib_path = lib_path.as_ref();
        let path = std::fs::canonicalize(lib_path).unwrap_or_else(|_| lib_path.to_path_buf());

        if let Some(start) = self.loading.iter().position(|loading| *loading == path) {
            let chain = self.loading[start..]
                .iter()
                .chain([&path])
                .map(|path| path.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(" -> ");
            let mut issues = IssueList::new();
            issues.push_err(ResolveErrorKind::CyclicDependency { chain });
            return Err(issues);
        }

        self.loading.push(path);
        let result = self.load_library_into_cache(lib_path);
        self.loading.pop();
        result
    }

    /// Load and resolve an external library and insert it into the library cache.
    fn load_library_into_cache(
        &mut self,
        lib_path: &std::path::Path,
    ) -> IssueResult<LibraryId, ResolveIssue> {
        let mut lib = Library::new();
        let mut lib_ctx = ResolveLibraryContext::new(&mut lib, self);
        let result = lib_ctx.load(lib_path);
        let mut issues = lib_ctx.to_issues();

        match result.and_then(|id| match id {
            Some(_) => self.lib_cache.write_unwrap().insert(lib).map(Some),
            None => Ok(None),
        }) {
            Ok(Some(lib_id)) => Ok((lib_id, issues)),
            // The library could not be compiled, its issues are recorded.
            Ok(None) => Err(issues),
            Err(err) => {
                issues.push_err(err);
                Err(issues)
            }
        }
    }

    /// Make a library in the cache an implicit dependency of every resolved library, e.g. `std`.
    pub fn add_implicit_dependency(&mut self, name: impl Into<Name>, lib_id: LibraryId) {
        self.implicit_dependencies.insert(name.into(), lib_id);
    }

    /// Set the symbols which can be used by name without `use`, e.g. `Circle` of `std`.
    pub fn set_prelude(&mut self, prelude: Prelude) {
        self.prelude = Some(prelude);
    }

    /// The symbols which can be used by name without `use`, if any.
    pub fn prelude(&self) -> Option<&Prelude> {
        self.prelude.as_ref()
    }
}

/// Builder for constructing a [`ResolveContext`].
#[derive(Debug, Default)]
pub struct ResolveContextBuilder {
    lib_search_paths: Vec<std::path::PathBuf>,
    lib_cache: Option<SharedMut<LibraryCache>>,
    src_cache: Option<SharedMut<SourceCache>>,
}

impl ResolveContextBuilder {
    /// Create a new `ResolveContextBuilder` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single library search path.
    pub fn with_search_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.lib_search_paths.push(path.into());
        self
    }

    /// Add multiple library search paths.
    pub fn with_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<std::path::PathBuf>,
    {
        self.lib_search_paths
            .extend(paths.into_iter().map(Into::into));
        self
    }

    /// Attach a shared library cache.
    pub fn with_lib_cache(mut self, lib_cache: SharedMut<LibraryCache>) -> Self {
        self.lib_cache = Some(lib_cache);
        self
    }

    /// Attach a shared source cache.
    pub fn with_src_cache(mut self, src_cache: SharedMut<SourceCache>) -> Self {
        self.src_cache = Some(src_cache);
        self
    }

    /// Build and initialize the [`ResolveContext`].
    pub fn build(self) -> ResolveContext {
        ResolveContext {
            _lib_search_paths: self.lib_search_paths,
            implicit_dependencies: BTreeMap::new(),
            lib_cache: self.lib_cache.unwrap_or_default(),
            src_cache: self.src_cache.unwrap_or_default(),
            loading: Vec::new(),
            prelude: None,
        }
    }
}
