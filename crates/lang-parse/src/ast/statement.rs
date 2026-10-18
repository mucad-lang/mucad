// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_macros::Visit;
use serde::Serialize;

use crate::ast;
use crate::ast::{Span, def};

/// An inner doc block
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct InnerDocComment {
    pub span: Span,
    pub line: String,
}

/// A µcad statement.
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
pub enum Statement {
    /// Workbench statement: `part Foo() { ... }`
    Workbench(def::Workbench),
    /// Inline Module: `mod foo { ... }`
    InlineModule(def::InlineModule),
    /// File Module: `mod foo;`
    FileModule(def::FileModule),
    /// Function statement: `fn bar() { ... }`
    Function(def::Function),
    /// Use statement: `use foo::bar;`
    Use(def::Use),
    /// Constant definition: `const FOO = 42mm`
    Const(def::Constant),
    /// Init definition: `init() { ... }`
    Init(Init),
    /// Return statement: `return 23mm;`
    Return(Return),
    /// Inner attribute: `#![...]`
    InnerAttribute(Attribute),
    /// Inner documentation: `//! Doc comment`
    InnerDocComment(InnerDocComment),
    /// Local assignment: `foo = bar;`
    LocalAssignment(LocalAssignment),
    /// Property: `prop bar = 42mm;`
    Property(PropertyAssignment),
    /// Expression statement: `foo | bar;`
    Expression(ExpressionStatement),
    /// Any error occured during parsing.
    Error(Span),
}

impl Statement {
    /// Get the span for the statement
    pub fn span(&self) -> Span {
        use Statement::*;

        match self {
            Workbench(st) => st.span.clone(),
            InlineModule(st) => st.span.clone(),
            FileModule(st) => st.span.clone(),
            Function(st) => st.span.clone(),
            Use(st) => st.span.clone(),
            Const(st) => st.span.clone(),
            Init(st) => st.span.clone(),
            Return(st) => st.span.clone(),
            InnerAttribute(st) => st.span.clone(),
            LocalAssignment(st) => st.span.clone(),
            Property(st) => st.span.clone(),
            Expression(st) => st.span.clone(),
            InnerDocComment(st) => st.span.clone(),
            Error(span) => span.clone(),
        }
    }

    /// Test if statement is supposed to end with a semicolon.
    pub fn ends_with_semicolon(&self) -> bool {
        match self {
            Statement::Workbench(_) => false,
            Statement::InlineModule(_) => false,
            Statement::Function(_) => false,
            Statement::InnerAttribute(_) => false,
            Statement::InnerDocComment(_) => false,
            Statement::Init(_) => false,
            Statement::Error(_) => false,

            Statement::Use(_) => true,
            Statement::Const(_) => true,
            Statement::Return(_) => true,
            Statement::FileModule(_) => true,
            Statement::LocalAssignment(_) => true,
            Statement::Property(_) => true,
            Statement::Expression(e) => {
                !matches!(&e.expr, ast::Expression::Body(_) | ast::Expression::If(_))
            }
        }
    }
}

/// An init definition for a workbench
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Init {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ast::ItemExtras,
    pub doc: DocBlock,
    pub attr: Attributes,
    pub parameters: ParameterList,
    pub body: ast::Body,
}

/// A return statement
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Return {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ast::ItemExtras,
    pub expr: Option<ast::Expression>,
}

/// A parameter list of a workbench definition or function definition
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ParameterList {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub parameters: Vec<Parameter>,
}

impl ast::Dummy for ParameterList {
    fn dummy(span: Span) -> Self {
        Self {
            span,
            extras: ast::ItemExtras::default(),
            parameters: Vec::default(),
        }
    }
}

/// A parameter for a workbench definition or function definition
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Parameter {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub doc: ast::DocBlock,
    pub attr: Attributes,
    pub id: ast::Identifier,
    pub ty: Option<ast::Type>,
    pub default: Option<ast::Expression>,
}

/// An attribute that can be attached to a statement
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Attribute {
    pub span: Span,
    #[visit(skip)]
    pub is_inner: bool,
    pub extras: ast::ItemExtras,
    pub commands: Vec<AttributeCommand>,
}

/// A list of attributes
#[derive(Debug, Hash, PartialEq, Serialize)]
pub struct Attributes(pub Vec<Attribute>);

/// The contents an an [`Attribute`]
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
pub enum AttributeCommand {
    /// A single identifier: `#[deprecated]`
    Ident(ast::Identifier),
    /// A meta data assignent: `#[color = RED]`
    Assignment(LocalAssignment),
    /// A call: `#[export("file.svg")`
    Call(ast::Call),
}

/// A local assignment: `a = 42`
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct LocalAssignment {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub attr: Attributes,
    pub id: ast::Identifier,
    pub ty: Option<ast::Type>,
    pub expr: Box<ast::Expression>,
}

/// A property assignment: `prop a = 42`
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct PropertyAssignment {
    pub span: Span,
    pub keyword_span: Span,
    pub extras: ast::ItemExtras,
    pub doc: DocBlock,
    pub attr: Attributes,
    pub id: ast::Identifier,
    pub ty: Option<ast::Type>,
    pub value: Box<ast::Expression>,
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize)]
#[allow(missing_docs)]
pub enum CommentInner {
    // A list of single line comments starting with `//`.
    SingleLine(String),
    // Comments embraced with `/* ... */`.
    MultiLine(String),
}

/// A single- or multi-line comment
#[derive(Debug, Clone, Hash, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct Comment {
    pub span: Span,
    pub inner: CommentInner,
}

/// Lines of inner or outer doc block including prefix `///`/`//!`.
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct DocBlock {
    pub span: Span,
    pub lines: Vec<String>,
}

/// A statement containing of a bare expression
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct ExpressionStatement {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub attr: Attributes,
    pub expr: ast::Expression,
}

/// A list of statements, with optional trailing whitespace kept and an optional "tail" expression
#[derive(Debug, Hash, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct StatementList {
    pub span: Span,
    pub extras: ast::ItemExtras,
    pub statements: Vec<(Statement, ast::TrailingExtras)>,
    pub tail: Option<Box<ExpressionStatement>>,
}

impl ast::Dummy for StatementList {
    fn dummy(span: Span) -> Self {
        Self {
            span,
            extras: ast::ItemExtras::default(),
            statements: Vec::default(),
            tail: None,
        }
    }
}
