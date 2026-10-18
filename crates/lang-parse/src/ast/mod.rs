// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod def;
mod expression;
mod literal;
mod statement;
mod ty;
pub mod visitor;

use mucad_lang_base::{Name, Span};

pub use expression::*;
pub use literal::*;
use mucad_macros::Visit;
use serde::Serialize;
pub use statement::*;
pub use ty::*;

pub use visitor::Visitor;

/// A µcad identifier
#[derive(Debug, PartialEq, Hash, Eq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct Identifier {
    pub span: Span,
    pub name: Name,
}

impl Dummy for Identifier {
    fn dummy(span: Span) -> Self {
        Self {
            span,
            name: Name::default(),
        }
    }
}
/// Whitespace
#[derive(Debug, Clone, Hash, PartialEq, Visit, Serialize)]
#[visit(default)]
pub struct Whitespace(pub String);

#[derive(Debug, Clone, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum ItemExtra {
    Comment(Comment),
    Whitespace(Whitespace),
}

/// Non-syntactic extras that can be attached to many ast nodes
#[derive(Clone, Debug, Hash, PartialEq, Default, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ItemExtras {
    pub leading: LeadingExtras,
    pub trailing: TrailingExtras,
}

/// Extras that occur *before* a syntax element.
#[derive(Debug, Clone, Hash, PartialEq, Default, Serialize)]
#[allow(missing_docs)]
pub struct TrailingExtras(pub Vec<ItemExtra>);

/// Extras that occur *after* a syntax element.
#[derive(Debug, Clone, Hash, PartialEq, Default, Serialize)]
#[allow(missing_docs)]
pub struct LeadingExtras(pub Vec<ItemExtra>);

/// Return a dummy of this syntax element.
///
/// Used for recovery.
pub(crate) trait Dummy {
    fn dummy(span: Span) -> Self;
}

/// A µcad abstract syntax tree
#[derive(Debug, Hash, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Source {
    pub span: Span,
    pub statements: StatementList,
}
