// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Check for unreachable initializers.

use mucad_lang_base::{IssueList, PushIssue};
use mucad_lang_types::Ty;

use super::{Check, CheckContext};
use crate::{
    ResolveErrorKind, ResolveIssue, SymbolDef,
    library::{
        Symbol,
        symbol::{Parameter, workbench::Init},
        visitor,
    },
};

/// Checks that each initializer of a workbench can be chosen.
///
/// The first matching initializer in definition order is chosen, the building plan last.
/// So an initializer is unreachable if an earlier one matches the same arguments.
#[derive(Default)]
pub(crate) struct InitCheck(IssueList<ResolveIssue>);

impl Check for InitCheck {
    /// Check the initializers of all workbenches of the library,
    /// an error for each unreachable initializer.
    fn check(ctx: &CheckContext) -> IssueList<ResolveIssue> {
        use visitor::Visitor;

        let mut init_check = Self::default();
        init_check.visit(ctx.lib);
        init_check.0
    }
}

/// Returns if `earlier` matches all arguments `init` matches with the same priority,
/// so `init` is never chosen.
///
/// This is the case if all parameters of `init` are parameters of `earlier`,
/// `earlier` requires no other parameters and its other parameters keep the short names unique.
fn shadows(earlier: &Init, init: &Init) -> bool {
    let first_letter = |param: &Parameter| param.id.as_str().chars().next();
    let in_init = |id| init.parameters.iter().find(|param| &param.id == id);

    let same_parameters = init.parameters.iter().all(|param| {
        earlier
            .parameters
            .iter()
            .any(|other| other.id == param.id && other.ty() == param.ty())
    });
    let no_other_required = earlier
        .parameters
        .iter()
        .filter(|param| param.default_value.is_none())
        .all(|param| in_init(&param.id).is_some_and(|param| param.default_value.is_none()));
    let unique_short_names = earlier
        .parameters
        .iter()
        .filter(|param| in_init(&param.id).is_none())
        .all(|other| {
            init.parameters
                .iter()
                .all(|param| first_letter(param) != first_letter(other))
        });

    same_parameters && no_other_required && unique_short_names
}

impl visitor::LeafVisitor for InitCheck {}
impl visitor::ConstantVisitor for InitCheck {}
impl visitor::WorkbenchExpressionVisitor for InitCheck {}
impl visitor::WorkbenchVisitor for InitCheck {}
impl visitor::FnVisitor for InitCheck {}
impl visitor::SourceVisitor for InitCheck {}

impl visitor::Visitor for InitCheck {
    fn visit_symbol(&mut self, symbol: &Symbol) {
        let SymbolDef::Workbench(workbench) = &symbol.def else {
            return;
        };

        let inits = &workbench.signature.inits;
        for (index, init) in inits.iter().enumerate() {
            if let Some(earlier) = inits[..index].iter().find(|earlier| shadows(earlier, init)) {
                self.0.push_err(ResolveErrorKind::UnreachableInit {
                    init: init.to_string(),
                    src_ref: init.parameters.src_ref,
                    earlier: earlier.to_string(),
                    earlier_src_ref: earlier.parameters.src_ref,
                });
            }
        }
    }
}
