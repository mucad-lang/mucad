// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    Desugar, LowerContext, LowerError, LowerResult,
    desugar::{attribute::outer_with_doc, for_each_statement},
    ir,
};

use mucad_lang_base::{PushIssue, SpanToSrcRef};
use mucad_lang_parse::ast;

impl Desugar<ast::def::FileModule> for ir::desugared::FileModule {
    fn desugar(node: &ast::def::FileModule, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            meta: ir::Meta {
                name: Some(ir::Identifier::desugar(&node.id, context)?),
                src_ref: context.span_to_src_ref(&node.span),
                vis: ir::Visibility::desugar(&node.vis, context)?,
                keyword_src_ref: context.span_to_src_ref(&node.keyword_span),
            },
            attr: outer_with_doc(&node.doc, &node.attr, context)?,
        })
    }
}

impl Desugar<ast::Statement> for Option<ir::desugared::FileModule> {
    fn desugar(stmt: &ast::Statement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match stmt {
            ast::Statement::FileModule(file_module) => {
                Some(ir::desugared::FileModule::desugar(file_module, context)?)
            }
            _ => None,
        })
    }
}

impl Desugar<ast::ExpressionStatement> for Option<ir::desugared::FileModule> {
    fn desugar(_stmt: &ast::ExpressionStatement, _context: &mut LowerContext) -> LowerResult<Self> {
        Ok(None)
    }
}

impl Desugar<ast::StatementList> for ir::desugared::InlineModuleItems {
    fn desugar(statements: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        for_each_statement(statements, context, |stmt, context| {
            let src_ref = context.span_to_src_ref(&stmt.span());
            use ast::Statement::*;
            match stmt {
                FileModule(_) | Return(_) | Expression(_) | LocalAssignment(_) | Property(_)
                | Init(_) | Error(_) => {
                    context.push_err(LowerError::StatementNotAllowed { src_ref })
                }
                _ => {}
            }
            Ok(())
        })?;

        Ok(Self {
            modules: Box::desugar(statements, context)?,
            aliases: ir::desugared::Aliases::desugar(statements, context)?,
            constants: Box::desugar(statements, context)?,
            functions: Box::desugar(statements, context)?,
            workbenches: Box::desugar(statements, context)?,
        })
    }
}

impl Desugar<ast::def::InlineModule> for ir::desugared::InlineModule {
    fn desugar(node: &ast::def::InlineModule, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            meta: ir::Meta {
                name: Some(ir::Identifier::desugar(&node.id, context)?),
                src_ref: context.span_to_src_ref(&node.span),
                vis: ir::Visibility::desugar(&node.vis, context)?,
                keyword_src_ref: context.span_to_src_ref(&node.keyword_span),
            },
            attr: crate::desugar::attribute::outer_with_doc(&node.doc, &node.attr, context)?
                .extend(ir::Attributes::desugar(&node.body.statements, context)?),
            items: ir::desugared::InlineModuleItems::desugar(&node.body.statements, context)?,
        })
    }
}

impl Desugar<ast::Statement> for Option<ir::desugared::InlineModule> {
    fn desugar(stmt: &ast::Statement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match stmt {
            ast::Statement::InlineModule(inline_module) => Some(
                ir::desugared::InlineModule::desugar(inline_module, context)?,
            ),
            _ => None,
        })
    }
}

impl Desugar<ast::ExpressionStatement> for Option<ir::desugared::InlineModule> {
    fn desugar(_: &ast::ExpressionStatement, _: &mut LowerContext) -> LowerResult<Self> {
        Ok(None)
    }
}
