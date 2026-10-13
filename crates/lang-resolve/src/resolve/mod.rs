// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The resolve compiler stage.
//!
//! TODO Doc

mod case_check;
mod context;
pub mod stack;
mod type_check;
pub mod visitor;

use microcad_lang_base::{IssueResult, PushIssue, Source};

use crate::{Library, ResolveIssue, ResolveResult, library::SymbolNodeId};

pub use context::{LibraryCache, ResolveContext, ResolveLibraryContext, SourceCache};

/// Resolve a single source file or a library directory.
pub fn resolve(
    path: impl AsRef<std::path::Path>,
    ctx: &mut ResolveContext,
) -> IssueResult<Library, ResolveIssue> {
    resolve_with(ctx, |lib_ctx| lib_ctx.load(path))
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
