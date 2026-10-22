// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Check for cyclic aliases.

use mucad_lang_base::{HashSet, IssueList, PushIssue, SymbolId};
use mucad_lang_lower::ir::Path;

use super::Check;
use crate::{
    Library, ResolveErrorKind, ResolveIssue, SymbolDef, SymbolNodeRef,
    library::{SymbolNodeId, visitor},
};

/// Checks that aliases stand for a symbol, which is not an alias of themselves,
/// e.g. `use d as c; use c as d;`.
#[derive(Default)]
pub(crate) struct AliasCheck(IssueList<ResolveIssue>);

impl Check for AliasCheck {
    /// Check all aliases of `library`, an error for each cyclic alias.
    fn check(library: &Library) -> IssueList<ResolveIssue> {
        use visitor::Visitor;

        let mut alias_check = Self::default();
        alias_check.visit(library);
        alias_check.0
    }
}

/// The alias of the library `node` stands for, if `node` is an alias of an alias.
fn target_alias(node: SymbolNodeRef<'_>) -> Option<SymbolNodeRef<'_>> {
    let SymbolDef::Alias(alias) = &node.get().def else {
        return None;
    };
    let Path::Resolved(SymbolId::Item(target)) = &alias.path else {
        return None;
    };
    let target = SymbolNodeRef::new(*target, node.arena);
    matches!(target.get().def, SymbolDef::Alias(_)).then_some(target)
}

impl visitor::LeafVisitor for AliasCheck {}
impl visitor::ConstantVisitor for AliasCheck {}
impl visitor::WorkbenchExpressionVisitor for AliasCheck {}
impl visitor::WorkbenchVisitor for AliasCheck {}
impl visitor::FnVisitor for AliasCheck {}
impl visitor::SourceVisitor for AliasCheck {}

impl visitor::Visitor for AliasCheck {
    /// An alias is cyclic, if following its targets never ends at a symbol other than an alias.
    fn visit_node(&mut self, node: SymbolNodeRef<'_>) {
        if !matches!(node.get().def, SymbolDef::Alias(_)) {
            return;
        }

        let mut visited: HashSet<SymbolNodeId> = HashSet::default();
        let mut current = node;
        while visited.insert(current.id) {
            match target_alias(current) {
                Some(target) => current = target,
                None => return,
            }
        }

        let symbol = node.get();
        self.0.push_err(ResolveErrorKind::CyclicAlias {
            name: symbol
                .meta
                .name
                .as_ref()
                .map(|name| name.to_string())
                .unwrap_or_default(),
            src_ref: symbol.meta.src_ref,
        });
    }
}
