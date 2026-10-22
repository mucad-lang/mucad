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

use crate::{Library, ResolveIssue};

/// A trait to check a library for issues.
pub(crate) trait Check {
    fn check(library: &Library) -> IssueList<ResolveIssue>;
}
