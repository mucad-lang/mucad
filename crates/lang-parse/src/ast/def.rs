// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Definition nodes in the AST that will eventually be resolved into Symbols.
//!
//! Each definition must have syntax elements in that order:
//! * A span
//! * Optional Item extras like comments
//! * Optional Doc block
//! * Attributes
//! * An optional visibility
//! * A keyword (with span)
//! * An `id` (except Use definitions)

use crate::ast::{
    Attributes, Body, DocBlock, Expression, Identifier, ItemExtras, ParameterList, Span, Type,
};

use mucad_lang_base::Spanned;
pub use mucad_lang_base::element::WorkbenchKind;

use mucad_macros::Visit;
use serde::Serialize;

/// An optional visibility modifier
///
/// it can be part of constant, module, function or workbench definitions.
#[derive(Debug, Hash, PartialEq, Clone, Visit, Serialize)]
#[visit(default)]
pub enum Visibility {
    /// `pub`
    Public,
}

#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Workbench {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ItemExtras,
    pub doc: DocBlock,
    pub kind: WorkbenchKind,
    pub attr: Attributes,
    pub vis: Option<Spanned<Visibility>>,
    pub id: Identifier,
    pub parameters: ParameterList,
    pub body: Body,
}

/// A definition of a module
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct InlineModule {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ItemExtras,
    pub doc: DocBlock,
    pub attr: Attributes,
    pub vis: Option<Spanned<Visibility>>,
    pub id: Identifier,
    pub body: Body,
}

/// A definition of a module
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct FileModule {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ItemExtras,
    pub doc: DocBlock,
    pub attr: Attributes,
    pub vis: Option<Spanned<Visibility>>,
    pub id: Identifier,
}

/// A definition of a function
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Function {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ItemExtras,
    pub doc: DocBlock,
    pub attr: Attributes,
    pub vis: Option<Spanned<Visibility>>,
    pub id: Identifier,
    pub parameters: ParameterList,
    pub return_type: Option<Type>,
    pub body: Body,
}

/// A use definition will become an alias or a wildcard.
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Use {
    pub span: Span,
    pub attr: Attributes,
    pub keyword_span: Span,
    pub extras: ItemExtras,
    pub vis: Option<Spanned<Visibility>>,
    pub name: UseName,
    pub use_as: Option<Identifier>,
}

/// The name of the item being imported
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct UseName {
    pub span: Span,
    pub prefix: Option<Span>,
    pub extras: ItemExtras,
    pub parts: Vec<UseStatementPart>,
}

/// The parts a [`UseName`] consists of, separated by `::`
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub enum UseStatementPart {
    Identifier(Identifier),
    Glob(Span),
    Error(Span),
}

/// A const assignment: `const A = 42` / `pub A = 32`
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Constant {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ItemExtras,
    pub doc: DocBlock,
    pub attr: Attributes,
    pub vis: Option<Spanned<Visibility>>,
    pub id: Identifier,
    pub ty: Option<Type>,
    pub expr: Box<Expression>,
}
