// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    impl_tokens,
    semantic_tokens::{SemanticTokens, TokenContext},
};

use super::{TokenModifier, TokenType};

use mu::ast;
use mucad_driver::prelude as mu;

impl_tokens!(ast::LocalAssignment => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    ctx.push_token(&self_.id.span, TokenType::VARIABLE, &[]);
    if let Some(ty) = self_.ty.as_ref() { ty.semantic_tokens(ctx) }
});

impl_tokens!(ast::AttributeCommand => |self_, ctx| {
    match &self_ {
        ast::AttributeCommand::Ident(id) => ctx.push_token(&id.span, TokenType::PROPERTY, &[]),
        ast::AttributeCommand::Assignment(local_assignment) => local_assignment.semantic_tokens(ctx),
        ast::AttributeCommand::Call(call) => call.semantic_tokens(ctx),
    }
});

impl_tokens!(ast::Attributes => |self_, ctx| {
    self_.0.iter().for_each(|attr| attr.semantic_tokens(ctx));
});

impl_tokens!(ast::Attribute => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.commands.iter().for_each(|command| command.semantic_tokens(ctx));
});

impl_tokens!(ast::DocBlock => |self_, ctx| {
    ctx.push_token(&self_.span, TokenType::COMMENT, &[TokenModifier::DOCUMENTATION]);
});

impl_tokens!(ast::Parameter => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    if let Some(expr) = self_.default.as_ref() { expr.semantic_tokens(ctx) }
    ctx.push_token(&self_.id.span, TokenType::PARAMETER, &[]);
    if let Some(ty) = self_.ty.as_ref() { ty.semantic_tokens(ctx) }
});

impl_tokens!(ast::ParameterList => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.parameters.iter().for_each(|param| param.semantic_tokens(ctx));
});

impl_tokens!(ast::Init => |self_, ctx| {
        self_.extras.semantic_tokens(ctx);
        self_.doc.semantic_tokens(ctx);
        self_.attr.semantic_tokens(ctx);
        ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
        self_.parameters.semantic_tokens(ctx);
        self_.body.semantic_tokens(ctx);
});

impl_tokens!(ast::Body => statements);
//impl_tokens!(ast::Visibility => TokenType::KEYWORD);

impl_tokens!(ast::def::Workbench => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.doc.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    // TODO: self_.visibility.as_ref().map(|vis| vis.semantic_tokens(ctx));
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    ctx.push_token(&self_.id.span, TokenType::FUNCTION, &[]);
    self_.parameters.semantic_tokens(ctx);
    self_.body.semantic_tokens(ctx);
});

impl_tokens!(ast::Return => |self_, ctx| {
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    if let Some(expr) = self_.expr.as_ref() { expr.semantic_tokens(ctx) }
});

impl_tokens!(ast::def::InlineModule => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.doc.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    // TODO: self_.visibility.as_ref().map(|vis| vis.semantic_tokens(ctx));
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    ctx.push_token(&self_.id.span, TokenType::NAMESPACE, &[]);
    self_.body.semantic_tokens(ctx);
});

impl_tokens!(ast::def::FileModule => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.doc.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    // TODO: self_.visibility.as_ref().map(|vis| vis.semantic_tokens(ctx));
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    ctx.push_token(&self_.id.span, TokenType::NAMESPACE, &[]);
});

impl_tokens!(ast::def::Function => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.doc.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    // TODO: self_.visibility.as_ref().map(|vis| vis.semantic_tokens(ctx));
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    ctx.push_token(&self_.id.span, TokenType::FUNCTION, &[]);
    self_.parameters.semantic_tokens(ctx);
    self_.body.semantic_tokens(ctx);
});

impl_tokens!(ast::def::UseStatementPart => |self_, ctx| {
    match &self_ {
        ast::def::UseStatementPart::Identifier(id) => ctx.push_token(&id.span, TokenType::NAMESPACE, &[]),
        ast::def::UseStatementPart::Glob(span) => ctx.push_token(span, TokenType::OPERATOR, &[]),
        _ => {}
    }
});

impl_tokens!(ast::def::UseName => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.parts.iter().for_each(|part| part.semantic_tokens(ctx));
});

impl_tokens!(ast::def::Use => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    // TODO: self_.visibility.as_ref().map(|vis| vis.semantic_tokens(ctx));
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    self_.name.semantic_tokens(ctx);
    if let Some(as_) = self_.use_as.as_ref() { ctx.push_token(&as_.span, TokenType::NAMESPACE, &[]) }
});

impl_tokens!(ast::def::Constant => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.doc.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    // TODO: self_.visibility.as_ref().map(|vis| vis.semantic_tokens(ctx));
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    ctx.push_token(&self_.id.span, TokenType::PROPERTY, &[]);
    if let Some(ty) = self_.ty.as_ref() { ty.semantic_tokens(ctx) }
    self_.expr.semantic_tokens(ctx);
});

impl_tokens!(ast::PropertyAssignment => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    ctx.push_token(&self_.keyword_span, TokenType::KEYWORD, &[]);
    ctx.push_token(&self_.id.span, TokenType::PROPERTY, &[]);
    if let Some(ty) = self_.ty.as_ref() { ty.semantic_tokens(ctx) }
    self_.value.semantic_tokens(ctx);
});

impl_tokens!(ast::InnerDocComment => |self_, ctx| {
    ctx.push_token(&self_.span, TokenType::COMMENT, &[TokenModifier::DOCUMENTATION]);
});

impl_tokens!(ast::ExpressionStatement => |self_, ctx | {
    self_.extras.semantic_tokens(ctx);
    self_.attr.semantic_tokens(ctx);
    self_.expr.semantic_tokens(ctx);
});

impl_tokens!(ast::Statement => |self_, ctx| {
    match &self_ {
        Self::Workbench(workbench_definition) => workbench_definition.semantic_tokens(ctx),
        Self::InlineModule(inline_module) => inline_module.semantic_tokens(ctx),
        Self::FileModule(file_module) => file_module.semantic_tokens(ctx),
        Self::Function(function_definition) => function_definition.semantic_tokens(ctx),
        Self::Use(use_statement) => use_statement.semantic_tokens(ctx),
        Self::Const(const_assignment) => const_assignment.semantic_tokens(ctx),
        Self::Init(init_definition) => init_definition.semantic_tokens(ctx),
        Self::Return(ret) => ret.semantic_tokens(ctx),
        Self::InnerAttribute(attribute) => attribute.semantic_tokens(ctx),
        Self::InnerDocComment(inner_doc_comment) => inner_doc_comment.semantic_tokens(ctx),
        Self::LocalAssignment(local_assignment) => local_assignment.semantic_tokens(ctx),
        Self::Property(property_assignment) => property_assignment.semantic_tokens(ctx),
        Self::Expression(expression_statement) => expression_statement.semantic_tokens(ctx),
        Self::Error(_) => {}
    }
});

impl_tokens!(ast::StatementList => |self_, ctx| {
    self_.extras.semantic_tokens(ctx);
    self_.statements.iter().for_each(|(stmt, extras)| {
        stmt.semantic_tokens(ctx);
        extras.semantic_tokens(ctx);
    });
    if let Some(expr) = self_.tail.as_ref() { expr.semantic_tokens(ctx) }
});
