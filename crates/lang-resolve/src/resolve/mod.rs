// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The resolve compiler stage.
//!
//! TODO Doc

mod bind;
mod cache;
mod case_check;
mod loader;
pub mod stack;
mod type_check;
pub mod visitor;

use microcad_lang_base::{CompilationResult, HashId, IssueList, PushIssue, SharedMut};

use microcad_lang_lower::Ir;

use crate::{Library, ResolveError, ResolveInfo, ResolveIssue, ResolveWarning};

pub use crate::resolve::cache::{LibraryCache, SourceCache};

#[derive(Debug, Default)]
pub struct ResolveContext {
    lib_search_paths: Vec<std::path::PathBuf>,
    lib_cache: SharedMut<LibraryCache>,
    src_cache: SharedMut<SourceCache>,
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
    /// Get intermediate representation by hash id.
    pub fn get_ir(&self, id: HashId) -> Option<Ir> {
        self.src_cache
            .read_unwrap()
            .get(id)
            .and_then(|source_unit| source_unit.ir().cloned())
    }

    /// Source cache
    pub fn source_cache(&self) -> SharedMut<SourceCache> {
        self.src_cache.clone()
    }
}

/// A Resolve Context to resolve exactly one library at a time.
pub struct ResolveLibraryContext<'ctx, 'lib> {
    pub issues: IssueList<ResolveIssue>,
    pub lib: &'lib mut Library,
    pub ctx: &'ctx mut ResolveContext,
}

/// The scaffolding step builds the unresolved tree from a workspace directory.
///
/// The workspace can be resolved in two modes:
/// - `Lib`: We resolve the workspace as a library by loading the `lib.mu` file, if it exists.
///   If it does not exist, we load every file in the workspace root directory.
/// - `Source`: We resolve a single source file within the workspace and only load its dependencies.
///
/// Optionally, the workspace can contain a `mu.toml` manifest file.
/// The `mu.toml` contain metadata for the µcad library and defines external dependencies.
///
/// A workspace directory can look like this:
///
/// /my_project
/// ├── mu.toml # Optional, but mandatory for publishing.
/// ├── lib.mu # Optional. If it is not present and if we are in Lib mode, we load all files in the directory.
/// ├── foo.mu # A file in the workspace
/// ├── /foo # A subdirectory
/// |   ├── bar.mu
/// |   ... # More files
/// ├── baz.mu
impl<'ctx, 'lib> ResolveLibraryContext<'ctx, 'lib> {
    pub fn new(lib: &'lib mut Library, ctx: &'ctx mut ResolveContext) -> Self {
        Self {
            lib,
            ctx,
            issues: Default::default(),
        }
    }

    pub fn lib(&'lib self) -> &'lib Library {
        self.lib
    }

    pub fn ctx(&'ctx self) -> &'ctx ResolveContext {
        self.ctx
    }

    pub fn issues(self) -> IssueList<ResolveIssue> {
        self.issues
    }
}

impl<'ctx, 'lib> PushIssue<ResolveIssue> for ResolveLibraryContext<'ctx, 'lib> {
    fn push_issue(&mut self, issue: impl Into<ResolveIssue>) {
        self.issues.push_issue(issue);
    }

    fn push_err(&mut self, err: impl Into<ResolveError>) {
        self.issues.push_err(err);
    }

    fn push_warn(&mut self, warn: impl Into<ResolveWarning>) {
        self.issues.push_warn(warn);
    }

    fn push_info(&mut self, info: impl Into<ResolveInfo>) {
        self.issues.push_info(info);
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
            lib_search_paths: self.lib_search_paths,
            lib_cache: self.lib_cache.unwrap_or_default(),
            src_cache: self.src_cache.unwrap_or_default(),
        }
    }
}

pub fn resolve(
    path: impl AsRef<std::path::Path>,
    ctx: &mut ResolveContext,
) -> CompilationResult<Library, ResolveIssue> {
    let mut lib = Library::new();
    let mut lib_ctx = ResolveLibraryContext::new(&mut lib, ctx);

    match lib_ctx.load(path) {
        Ok(()) => {
            let issues = lib_ctx.issues();
            Ok((lib, issues))
        }
        Err(err) => {
            let mut issues = lib_ctx.issues();
            issues.push_err(err);
            Err(issues)
        }
    }
}
