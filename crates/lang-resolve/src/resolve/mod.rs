// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The resolve compiler stage.
//!
//! TODO Doc

mod alias_check;
mod case_check;
mod context;
mod duplicate_check;
mod init_check;
pub mod stack;
mod type_check;
pub mod visitor;

use mucad_lang_base::{HashSet, IssueResult, PushIssue, Source};

use crate::{Library, ResolveIssue, ResolveResult, library::SymbolNodeId, locate};

pub use context::{LibraryCache, Prelude, ResolveContext, ResolveLibraryContext, SourceCache};

/// Resolve a single source file or a library directory.
pub fn resolve(
    path: impl AsRef<std::path::Path>,
    ctx: &mut ResolveContext,
) -> IssueResult<Library, ResolveIssue> {
    resolve_with(ctx, |lib_ctx| lib_ctx.load(path))
}

/// All source files of the library, which contains the source file at `path`,
/// including the source files of all its dependencies.
///
/// The library is found with [`locate::library_root`] and resolved, which loads its dependencies.
/// Source files with errors are included.
pub fn library_source_files(
    path: impl AsRef<std::path::Path>,
    ctx: &mut ResolveContext,
) -> IssueResult<Vec<std::path::PathBuf>, ResolveIssue> {
    let (lib, issues) = resolve(locate::library_root(path), ctx)?;

    let lib_cache = ctx.lib_cache();
    let lib_cache = lib_cache.read_unwrap();
    let dependencies = lib
        .dependencies
        .values()
        .flat_map(|dep| lib_cache.source_files(dep));

    // Dependencies may share libraries.
    let mut seen = HashSet::default();
    let files = lib
        .source_files()
        .into_iter()
        .chain(dependencies)
        .filter(|path| seen.insert(path.clone()))
        .collect();

    Ok((files, issues))
}

/// Resolve a source, e.g. an editor buffer or stdin.
///
/// File modules are located relative to the source's path, if it has one.
pub fn resolve_source(
    source: impl Into<Source>,
    ctx: &mut ResolveContext,
) -> IssueResult<Library, ResolveIssue> {
    resolve_with(ctx, |lib_ctx| lib_ctx.load_source(source.into()))
}

/// Resolve a new library with `load`.
fn resolve_with(
    ctx: &mut ResolveContext,
    load: impl FnOnce(&mut ResolveLibraryContext) -> ResolveResult<Option<SymbolNodeId>>,
) -> IssueResult<Library, ResolveIssue> {
    let mut lib = Library::new();
    let mut lib_ctx = ResolveLibraryContext::new(&mut lib, ctx);

    let result = load(&mut lib_ctx);
    let mut issues = lib_ctx.to_issues();
    match result {
        Ok(Some(_)) => Ok((lib, issues)),
        // The source could not be compiled, its issues are recorded.
        Ok(None) => Err(issues),
        Err(err) => {
            issues.push_err(err);
            Err(issues)
        }
    }
}
