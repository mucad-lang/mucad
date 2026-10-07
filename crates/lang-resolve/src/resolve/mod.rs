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

use microcad_lang_base::{CompilationResult, PushIssue};

use crate::{Library, ResolveIssue};

pub use context::{LibraryCache, ResolveContext, ResolveLibraryContext, SourceCache};

pub fn resolve(
    path: impl AsRef<std::path::Path>,
    ctx: &mut ResolveContext,
) -> CompilationResult<Library, ResolveIssue> {
    let mut lib = Library::new();
    let mut lib_ctx = ResolveLibraryContext::new(&mut lib, ctx);

    match lib_ctx.load(path) {
        Ok(()) => {
            let issues = lib_ctx.to_issues();
            Ok((lib, issues))
        }
        Err(err) => {
            let mut issues = lib_ctx.to_issues();
            issues.push_err(err);
            Err(issues)
        }
    }
}
