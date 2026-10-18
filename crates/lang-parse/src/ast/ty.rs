// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ast;

use mucad_lang_base::{Name, Span};
use mucad_macros::Visit;
use serde::Serialize;

/// The possible types
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub enum Type {
    Single(SingleType),
    List(ListType),
    Tuple(TupleType),
}

impl Type {
    /// Get the span of the type
    pub fn span(&self) -> Span {
        match self {
            Type::Single(ty) => ty.span.clone(),
            Type::List(ty) => ty.span.clone(),
            Type::Tuple(ty) => ty.span.clone(),
        }
    }
}

impl ast::Dummy for Type {
    fn dummy(span: Span) -> Self {
        Type::Single(SingleType {
            span,
            name: Name::default(),
        })
    }
}

/// A type for a single numeric value
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct SingleType {
    pub span: Span,
    pub name: Name,
}

/// A list type
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ListType {
    pub span: Span,
    pub inner: Box<Type>,
}

/// A tuple type
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct TupleType {
    pub span: Span,
    pub inner: Vec<(Option<ast::Identifier>, Type)>,
}

/// A µcad unit: mm, m³, %.
#[derive(Debug, Hash, PartialEq, Eq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct Unit {
    pub span: Span,
    pub name: Name,
}
