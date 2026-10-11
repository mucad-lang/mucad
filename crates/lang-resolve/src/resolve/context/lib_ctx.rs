// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::{IssueList, PushIssue, SharedMut};

use crate::{
    Library, LibraryCache, ResolveContext, ResolveError, ResolveInfo, ResolveIssue, ResolveWarning,
    library::SymbolArena,
};

/// A Resolve Context to resolve exactly one library at a time.
pub struct ResolveLibraryContext<'ctx, 'lib> {
    issues: IssueList<ResolveIssue>,
    lib: &'lib mut Library,
    ctx: &'ctx mut ResolveContext,
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

    /// Immutable view of the library being resolved.
    pub fn lib(&self) -> &Library {
        self.lib
    }

    /// Mutable access to the library being resolved.
    pub fn lib_mut(&mut self) -> &mut Library {
        self.lib
    }

    pub fn ctx(&self) -> &ResolveContext {
        self.ctx
    }

    pub fn ctx_mut(&mut self) -> &mut ResolveContext {
        self.ctx
    }

    pub(crate) fn lib_cache(&self) -> SharedMut<LibraryCache> {
        self.ctx.lib_cache.clone()
    }

    pub(crate) fn arena_mut(&mut self) -> &mut SymbolArena {
        &mut self.lib.arena
    }

    pub fn to_issues(self) -> IssueList<ResolveIssue> {
        self.issues
    }

    /// Add the implicit dependencies of the context (e.g. `std`) to the library,
    /// unless its manifest says `no_std`.
    pub(crate) fn add_implicit_dependencies(&mut self) {
        if self.lib.no_std() {
            return;
        }
        let dependencies = self.ctx.implicit_dependencies().clone();
        self.lib.dependencies.extend(dependencies);
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
