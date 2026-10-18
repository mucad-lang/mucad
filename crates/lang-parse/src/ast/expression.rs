// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Name, Spanned};
use mucad_macros::Visit;
use serde::Serialize;

use crate::ast;
use crate::ast::Span;
use std::num::ParseIntError;

pub use mucad_lang_base::element::{BinaryOperator, UnaryOperator};

impl ast::visitor::Visit for BinaryOperator {}
impl ast::visitor::Visit for UnaryOperator {}

/// Any expression.
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
pub enum Expression {
    /// A literal: `42mm`
    Literal(ast::Literal),
    /// Something in `()` brackets: `(42mm)`
    Bracketed(Box<Expression>, Span),
    /// A tuple: `(a = 1, b = 23)`
    Tuple(TupleExpression),
    /// A range expression: `[1..4]`
    Range(RangeExpression),
    /// A list expression: `[1, 2, 3]`
    List(ListExpression),
    /// A format string: `"We have {n} items"`
    String(FormatString),
    /// A symbol path: `foo::bar::baz`
    SymbolPath(SymbolPath),
    /// A marker expression: `@input`
    Marker(ast::Identifier),
    /// A binary operation: `1 + 3`
    BinaryOperation(BinaryOperation),
    /// A unary operation: `-2`
    UnaryOperation(UnaryOperation),
    /// A body expression containing statements: `{ ... }`
    Body(ast::Body),
    /// A call: `call::me(1, 2, 3)`
    Call(Call),
    /// Accessing an element: `.foo`, `.rotate()`, `#attr`, `[1]`
    ElementAccess(ElementAccess),
    /// An if expression: `if a == b { ... } else { ... }`
    If(If),
    /// Any occurred during parsing
    Error(Span),
}

impl Expression {
    /// Get the source span for the identifier
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(ex) => ex.span.clone(),
            Expression::Bracketed(_, span) => span.clone(),
            Expression::Tuple(ex) => ex.span.clone(),
            Expression::Range(ex) => ex.span.clone(),
            Expression::List(ex) => ex.span.clone(),
            Expression::String(ex) => ex.span.clone(),
            Expression::SymbolPath(ex) => ex.span.clone(),
            Expression::Marker(ex) => ex.span.clone(),
            Expression::BinaryOperation(ex) => ex.span.clone(),
            Expression::UnaryOperation(ex) => ex.span.clone(),
            Expression::Body(ex) => ex.span.clone(),
            Expression::Call(ex) => ex.span.clone(),
            Expression::ElementAccess(ex) => ex.span.clone(),
            Expression::If(ex) => ex.span.clone(),
            Expression::Error(span) => span.clone(),
        }
    }

    /// Can this expression also be used as a statement, without extra semicolon
    pub fn is_also_statement(&self) -> bool {
        matches!(self, Expression::Body(_) | Expression::If(_))
    }

    /// Returns the single identifier of an expression, e.g. `r` in `-r` or `r * 2%`,
    /// but not in `r + w` or `[1..r] / r`, because the identifier must appear only once.
    pub fn single_identifier(&self) -> Option<Name> {
        use ast::visitor::Visit;

        let name = self.operand_identifier()?;
        let mut symbol_paths = SymbolPaths::default();
        let _ = self.visit(&mut symbol_paths);
        let occurrences = symbol_paths
            .0
            .iter()
            .filter(|symbol_path| matches!(symbol_path.parts.as_slice(), [id] if id.name == name))
            .count();
        (occurrences == 1).then_some(name)
    }

    /// The identifier, which is the only operand of operators with an identifier,
    /// e.g. `r` in `-r` or `r * 2%`, but not in `r + w` or `r.x`.
    fn operand_identifier(&self) -> Option<Name> {
        match self {
            Expression::Bracketed(expression, _) => expression.operand_identifier(),
            Expression::BinaryOperation(binary_operation) => match (
                binary_operation.lhs.operand_identifier(),
                binary_operation.rhs.operand_identifier(),
            ) {
                (Some(id), None) | (None, Some(id)) => Some(id),
                _ => None, // Both have identifiers (e.g., r + w) or neither does
            },
            Expression::UnaryOperation(unary_operation) => unary_operation.rhs.operand_identifier(),
            Expression::SymbolPath(symbol_path) => match symbol_path.parts.as_slice() {
                [identifier] => Some(identifier.name.clone()),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Collects the symbol paths of an expression, e.g. `n` twice in `[1..n] / n`.
#[derive(Default)]
struct SymbolPaths<'ast>(Vec<&'ast SymbolPath>);

impl<'ast> ast::visitor::Visitor<'ast> for SymbolPaths<'ast> {
    type BreakTy = ();

    fn visit_symbol_path(&mut self, node: &'ast SymbolPath) -> std::ops::ControlFlow<()> {
        self.0.push(node);
        std::ops::ControlFlow::Continue(())
    }
}

/// A string containing a format expression
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct FormatString {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub parts: Vec<StringPart>,
}

/// A part of a [`FormatString`]
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub enum StringPart {
    Char(StringCharacter),
    Content(ast::StringLiteral),
    Expression(StringExpression),
}

/// A single character that is part of a [`FormatString`]
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct StringCharacter {
    pub span: Span,
    pub character: char,
}

/// A format expression that is part of a [`FormatString`]
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct StringExpression {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub expr: Box<Expression>,
    #[serde(skip)] // TODO: Remove this skip
    pub specification: Box<StringFormatSpecification>,
}

/// The format specification for a [`StringExpression`], specifying the width and precision for number formatting
///
/// All parts of the specification are optional
#[derive(Debug, PartialEq, Visit)]
#[allow(missing_docs)]
#[visit(default)]
pub struct StringFormatSpecification {
    pub span: Span,
    pub precision: Option<Result<u32, (ParseIntError, Span)>>,
    pub width: Option<Result<u32, (ParseIntError, Span)>>,
}

impl std::hash::Hash for StringFormatSpecification {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        //
    }
}

impl StringFormatSpecification {
    /// Check if an part of the specification is specified
    pub fn is_some(&self) -> bool {
        self.precision.is_some() || self.width.is_some()
    }
}

/// An item that is part of a tuple expression
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct TupleItem {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub id: Option<ast::Identifier>,
    pub expr: Expression,
}

impl ast::Dummy for TupleItem {
    fn dummy(span: Span) -> Self {
        Self {
            span: span.clone(),
            extras: ast::ItemExtras::default(),
            id: None,
            expr: Expression::Error(span),
        }
    }
}

/// A tuple expression, a fixed size set of items that don't need to be the same type
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct TupleExpression {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub values: Vec<TupleItem>,
}

/// A range, containing all values from the start value (inclusive) till then end value (exclusive)
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct RangeExpression {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub start: Box<ListItem>,
    pub end: Box<ListItem>,
    pub unit: Option<ast::Unit>,
}

/// A list of items
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ListExpression {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub items: Vec<ListItem>,
    pub unit: Option<ast::Unit>,
}

/// An item that can be part of an list expression
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ListItem {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub expr: Expression,
}

/// A qualified name, containing one or more [`Identifier`]s separated by `::`
#[derive(Debug, Hash, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct SymbolPath {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub prefix: Option<Span>,
    pub parts: Vec<ast::Identifier>,
}

impl ast::visitor::Visit for SymbolPath {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> std::ops::ControlFlow<V::BreakTy>
    where
        V: ast::visitor::Visitor<'ast> + ?Sized,
    {
        visitor.visit_symbol_path(self)
    }
}

impl From<ast::Identifier> for SymbolPath {
    fn from(id: ast::Identifier) -> Self {
        let span = id.span.clone();
        Self {
            span,
            extras: ast::ItemExtras::default(),
            prefix: None,
            parts: vec![id],
        }
    }
}

/// A binary operation
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct BinaryOperation {
    pub span: Span,
    pub lhs: Box<Expression>,
    pub op: Spanned<BinaryOperator>,
    pub rhs: Box<Expression>,
}

/// A unary operation
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct UnaryOperation {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub op: Spanned<UnaryOperator>,
    pub rhs: Box<Expression>,
}

/// A function call
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Call {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub path: SymbolPath,
    pub arguments: ArgumentList,
}

/// An expression that access an element from another expression.
///
/// Either accessing an list or tuple item, accessing an attribute of a value or a method call.
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ElementAccess {
    pub span: Span,
    pub expr: Box<Expression>,
    pub element_chain: Vec<Element>,
}

/// The possible element access types
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub enum ElementInner {
    Attribute(ast::Identifier),
    Tuple(ast::Identifier),
    Method(Box<Call>),
    ListElement(Box<Expression>),
}

#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Element {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub inner: ElementInner,
}

#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Body {
    pub span: Span,
    pub statements: ast::StatementList,
}

/// An if expression, can be used as either a statement or expression
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct If {
    pub span: Span,
    pub if_span: Span,
    pub extras: ast::ItemExtras,
    pub condition: Box<Expression>,
    pub body: Body,
    pub next_if_span: Option<Span>,
    pub next_if: Option<Box<If>>,
    pub else_span: Option<Span>,
    pub else_body: Option<Body>,
}

/// A list of arguments to a function call
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ArgumentList {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub arguments: Vec<Argument>,
}

impl ast::Dummy for ArgumentList {
    fn dummy(span: Span) -> Self {
        Self {
            span,
            extras: ast::ItemExtras::default(),
            arguments: Vec::new(),
        }
    }
}

/// A function argument that is part of an [`ArgumentList`]
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub enum Argument {
    Unnamed(UnnamedArgument),
    Named(NamedArgument),
}

impl Argument {
    /// The name of the argument, if specified
    pub fn name(&self) -> Option<&ast::Identifier> {
        match self {
            Argument::Unnamed(_) => None,
            Argument::Named(arg) => Some(&arg.id),
        }
    }

    /// The value of the argument
    pub fn value(&self) -> &Expression {
        match self {
            Argument::Unnamed(arg) => &arg.expr,
            Argument::Named(arg) => &arg.expr,
        }
    }

    /// The span of the argument
    pub fn span(&self) -> &Span {
        match self {
            Argument::Unnamed(arg) => &arg.span,
            Argument::Named(arg) => &arg.span,
        }
    }
}

/// An argument without specified name
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct UnnamedArgument {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub expr: Expression,
}

/// An argument with a specified name
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct NamedArgument {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub id: ast::Identifier,
    pub expr: Expression,
}
