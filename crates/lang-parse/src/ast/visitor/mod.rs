// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Visitor for an abstract syntax tree.

mod collector;
mod visit;

pub use visit::Visit;

pub use collector::{
    CommentCollector, ExpectedDiagnostic, ExpectedDiagnostics, ExpectedDiagnosticsCollector,
    collect_expected_diagnostics,
};

use std::ops::ControlFlow;

use crate::ast;

/// Creates the default visit function implementation for a particular type
macro_rules! define_visit {
    ($fn_name:ident, $type_name:path) => {
        #[doc = concat!("Visits a `", stringify!($type_name), "` with this visitor")]
        fn $fn_name(&mut self, node: &'ast $type_name) -> std::ops::ControlFlow<Self::BreakTy> {
            node.visit(self)
        }
    };
}

/// Interface of the Visitor to be implemented.
///
/// TODO: Add more `visit_*` methods here.
pub trait Visitor<'ast> {
    /// Type which will be propagated from the visitor if completing early.
    type BreakTy;

    define_visit!(visit_comment, ast::Comment);

    /// Visits a `ast::SymbolPath` with this visitor, e.g. `foo::bar` or `x`.
    fn visit_symbol_path(&mut self, node: &'ast ast::SymbolPath) -> ControlFlow<Self::BreakTy> {
        node.extras.visit(self)?;
        node.parts.visit(self)
    }
}
