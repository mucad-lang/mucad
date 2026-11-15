// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Check the uses of deprecated and experimental symbols.

use mucad_lang_base::{IssueList, PushIssue, SrcRef, Stability, SymbolId};
use mucad_lang_lower::ir::Path;

use super::{Check, CheckContext};
use crate::{
    ResolveErrorKind, ResolveIssue, ResolveWarning, SymbolNodeExt, SymbolNodeRef, library::visitor,
};

/// Checks the resolved paths of the library:
/// - a deprecated symbol is a warning, but not inside a deprecated symbol, like in Rust,
/// - an experimental symbol of another library, e.g. of `std`, is a warning, or an error, if
///   experimental symbols are denied, see [`CheckContext::deny_experimental`].
pub(crate) struct StabilityCheck<'a> {
    ctx: &'a CheckContext<'a>,
    issues: IssueList<ResolveIssue>,
    /// Whether the visited symbol or a module around it is deprecated.
    in_deprecated: bool,
}

impl Check for StabilityCheck<'_> {
    /// Check all paths of the library, an issue for each use of a deprecated or experimental
    /// symbol, once for each position, see [`IssueList`].
    fn check(ctx: &CheckContext) -> IssueList<ResolveIssue> {
        use visitor::Visitor;

        let mut stability_check = StabilityCheck {
            ctx,
            issues: IssueList::default(),
            in_deprecated: false,
        };
        stability_check.visit(ctx.lib);
        stability_check.issues
    }
}

impl StabilityCheck<'_> {
    /// The stability and the full name of the symbol `id` of this or another library,
    /// e.g. `std::geo2d::Circle`, `None` for locals and built-ins.
    fn stability(&self, id: &SymbolId) -> Option<(Stability, String)> {
        let of = |node: SymbolNodeRef| {
            (
                node.get().ver.stability.clone(),
                node.abs_path().to_string(),
            )
        };
        match id {
            SymbolId::Item(node_id) => Some(of(SymbolNodeRef::new(*node_id, &self.ctx.lib.arena))),
            SymbolId::External(external_id) => {
                let lib = self.ctx.lib_cache.get(&external_id.lib_id)?;
                Some(of(SymbolNodeRef::new(external_id.id, &lib.arena)))
            }
            SymbolId::Builtin(_) | SymbolId::Local(_) => None,
        }
    }

    /// Report the use of the symbol `id` at `src_ref`, if it is deprecated or experimental.
    fn check_use(&mut self, id: &SymbolId, src_ref: SrcRef) {
        let Some((stability, path)) = self.stability(id) else {
            return;
        };
        match stability {
            Stability::Deprecated { .. } if !self.in_deprecated => {
                self.issues.push_warn(ResolveWarning::Deprecated {
                    path,
                    deprecation: stability.deprecation().unwrap_or_default(),
                    note: stability.deprecation_note().map(str::to_string),
                    src_ref,
                });
            }
            // The own experimental symbols can be used.
            Stability::Experimental if matches!(id, SymbolId::External(_)) => {
                if self.ctx.deny_experimental {
                    self.issues
                        .push_err(ResolveErrorKind::Experimental { path, src_ref });
                } else {
                    self.issues
                        .push_warn(ResolveWarning::Experimental { path, src_ref });
                }
            }
            _ => {}
        }
    }
}

impl visitor::LeafVisitor for StabilityCheck<'_> {
    /// Only paths written in a source have a position.
    fn visit_path(&mut self, path: &Path) {
        if let Path::Resolved { id, src_ref } = path
            && src_ref.is_some()
        {
            self.check_use(id, *src_ref);
        }
    }
}

impl visitor::ConstantVisitor for StabilityCheck<'_> {}
impl visitor::WorkbenchExpressionVisitor for StabilityCheck<'_> {}
impl visitor::WorkbenchVisitor for StabilityCheck<'_> {}
impl visitor::FnVisitor for StabilityCheck<'_> {}
impl visitor::SourceVisitor for StabilityCheck<'_> {}

impl visitor::Visitor for StabilityCheck<'_> {
    fn visit_node(&mut self, node: SymbolNodeRef<'_>) {
        self.in_deprecated = std::iter::once(node)
            .chain(node.ancestors())
            .any(|node| node.get().ver.stability.is_deprecated());
        self.visit_symbol(node.get());
    }
}
