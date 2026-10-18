// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerResult, ir};

use mucad_lang_base::SpanToSrcRef;
use mucad_lang_parse::ast;

impl Desugar<ast::def::Constant> for ir::desugared::Constant {
    fn desugar(node: &ast::def::Constant, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            meta: ir::Meta {
                name: Some(ir::Identifier::desugar(&node.id, context)?),
                src_ref: context.span_to_src_ref(&node.span),
                vis: ir::Visibility::desugar(&node.vis, context)?,
                keyword_src_ref: context.span_to_src_ref(&node.keyword_span),
            },
            attr: crate::desugar::attribute::outer_with_doc(&node.doc, &node.attr, context)?,
            ty: ir::Type::desugar(&node.ty, context)?,
            expr: ir::ConstantExpression::desugar(node.expr.as_ref(), context)?,
        })
    }
}

impl Desugar<ast::Statement> for Option<ir::desugared::Constant> {
    fn desugar(stmt: &ast::Statement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match stmt {
            ast::Statement::Const(const_assignment) => {
                Some(ir::desugared::Constant::desugar(const_assignment, context)?)
            }
            _ => None,
        })
    }
}

impl Desugar<ast::ExpressionStatement> for Option<ir::desugared::Constant> {
    fn desugar(_: &ast::ExpressionStatement, _: &mut LowerContext) -> LowerResult<Self> {
        Ok(None)
    }
}
