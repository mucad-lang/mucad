// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod alias_check;
mod case_check;
mod duplicate_check;
mod init_check;
mod type_check;

pub(crate) use alias_check::AliasCheck;
pub(crate) use case_check::CaseCheck;
pub(crate) use duplicate_check::DuplicateCheck;
pub(crate) use init_check::InitCheck;
use mucad_lang_base::IssueList;
pub(crate) use type_check::{TypeCheck, infer_parameter_types};

use mucad_builtin::BuiltinLibrary;

use crate::{Library, LibraryCache, ResolveIssue, TypeContext};

/// The library to check, with the symbols it can use: of other libraries and built-ins.
pub(crate) struct CheckContext<'a> {
    /// The library to check.
    pub(crate) lib: &'a Library,
    /// The other libraries, e.g. `std`.
    pub(crate) lib_cache: &'a LibraryCache,
    pub(crate) builtins: &'a BuiltinLibrary,
}

impl<'a> CheckContext<'a> {
    /// A context to derive the types of expressions of the library.
    pub(crate) fn types(&self) -> TypeContext<'a> {
        TypeContext::new(self.lib, self.builtins).with_lib_cache(self.lib_cache)
    }
}

/// A trait to check a library for issues.
pub(crate) trait Check {
    fn check(ctx: &CheckContext) -> IssueList<ResolveIssue>;
}
