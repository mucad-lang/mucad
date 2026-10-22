// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Check the case of symbol names.

use mucad_lang_base::{
    IssueList, PushIssue, SrcReferrer,
    element::{Case, WorkbenchKind},
};

use super::Check;
use crate::{
    Library, ResolveIssue, ResolveWarning, SymbolDef,
    library::{Symbol, visitor},
};

/// Checks that symbol names are written in the case of their kind,
/// e.g. `UPPER_SNAKE_CASE` for constants.
#[derive(Default)]
pub(crate) struct CaseCheck(IssueList<ResolveIssue>);

impl Check for CaseCheck {
    /// Check the names of all symbols of `library`, a warning for each name in a wrong case.
    fn check(library: &Library) -> IssueList<ResolveIssue> {
        use visitor::Visitor;

        let mut case_check = Self::default();
        case_check.visit(library);
        case_check.0
    }
}

/// The case of the name of a symbol, e.g. `UPPER_SNAKE_CASE` for constants.
fn expected_case(def: &SymbolDef) -> Option<Case> {
    use SymbolDef::*;
    match def {
        InlineModule(_) | SourceFile(_) | Function(_) => Some(Case::LowerSnake),
        Workbench(workbench) => match workbench.signature.kind {
            WorkbenchKind::Sketch | WorkbenchKind::Part => Some(Case::Pascal),
            WorkbenchKind::Op => Some(Case::LowerSnake),
        },
        Constant(_) => Some(Case::UpperSnake),
        Alias(_) | Wildcard(_) | Root(_) => None,
    }
}

impl visitor::LeafVisitor for CaseCheck {}
impl visitor::ConstantVisitor for CaseCheck {}
impl visitor::WorkbenchExpressionVisitor for CaseCheck {}
impl visitor::WorkbenchVisitor for CaseCheck {}
impl visitor::FnVisitor for CaseCheck {}
impl visitor::SourceVisitor for CaseCheck {}

impl visitor::Visitor for CaseCheck {
    /// Only names written in code are checked, not those taken from a file name,
    /// e.g. of the main source file.
    fn visit_symbol(&mut self, symbol: &Symbol) {
        if let (Some(name), Some(expected)) = (&symbol.meta.name, expected_case(&symbol.def))
            && name.src_ref().is_some()
            && !name.is_case(expected)
        {
            self.0.push_warn(ResolveWarning::WrongCase {
                name: name.clone(),
                expected,
                src_ref: name.src_ref(),
            });
        }
    }
}
