// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Check for symbols defined twice.

use mucad_lang_base::{HashSet, Identifier, SrcReferrer};

use crate::{Library, ResolveError, ResolveErrorKind, SymbolNodeRef, library::visitor};

/// Checks that the symbols of a module or workbench have unique names,
/// e.g. no two constants `A` in the same module.
#[derive(Default)]
pub(crate) struct DuplicateCheck(Vec<ResolveError>);

impl DuplicateCheck {
    /// Check the symbols of `library`, returns an error for each symbol defined again.
    pub(crate) fn check(library: &Library) -> Vec<ResolveError> {
        use visitor::Visitor;

        let mut duplicate_check = Self::default();
        duplicate_check.visit(library);
        duplicate_check.0
    }
}

impl visitor::LeafVisitor for DuplicateCheck {}
impl visitor::ConstantVisitor for DuplicateCheck {}
impl visitor::WorkbenchExpressionVisitor for DuplicateCheck {}
impl visitor::WorkbenchVisitor for DuplicateCheck {}
impl visitor::FnVisitor for DuplicateCheck {}
impl visitor::SourceVisitor for DuplicateCheck {}

impl visitor::Visitor for DuplicateCheck {
    /// The children of `node` with the name of an earlier child are defined again.
    fn visit_node(&mut self, node: SymbolNodeRef<'_>) {
        // Identifiers are equal without their source location, so the set holds the first names.
        let mut first: HashSet<&Identifier> = HashSet::default();
        for name in node
            .children()
            .filter_map(|child| child.get().meta.name.as_ref())
        {
            match first.get(name) {
                Some(first_name) => self.0.push(
                    ResolveErrorKind::SymbolAlreadyDefined {
                        name: name.to_string(),
                        src_ref: name.src_ref(),
                        first_src_ref: first_name.src_ref(),
                    }
                    .into(),
                ),
                None => {
                    first.insert(name);
                }
            }
        }
    }
}
