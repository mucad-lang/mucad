// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerError, LowerResult, desugar::for_each_statement, ir};

use mucad_lang_base::{Identifier, PushIssue, SpanToSrcRef, SrcRef, element::Visibility};
use mucad_lang_parse::ast;

impl Desugar<ast::StatementList> for ir::desugared::SourceItems {
    fn desugar(statements: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        for_each_statement(statements, context, |stmt, context| {
            let src_ref = context.span_to_src_ref(&stmt.span());
            match stmt {
                ast::Statement::Init(_)
                | ast::Statement::Return(_)
                | ast::Statement::Property(_)
                | ast::Statement::Error(_) => {
                    context.push_err(LowerError::StatementNotAllowed { src_ref })
                }
                _ => {}
            }
            Ok(())
        })?;

        Ok(Self {
            file_modules: Box::desugar(statements, context)?,
            inline_modules: Box::desugar(statements, context)?,
            aliases: ir::desugared::Aliases::desugar(statements, context)?,
            constants: Box::desugar(statements, context)?,
            functions: Box::desugar(statements, context)?,
            workbenches: Box::desugar(statements, context)?,
        })
    }
}

impl Desugar<ast::Attributes> for Box<[ir::ExportAttribute]> {
    fn desugar(node: &ast::Attributes, context: &mut LowerContext) -> LowerResult<Self> {
        ir::Attributes::desugar(node, context)?.fetch_exports()
    }
}

impl Desugar<ast::ExpressionStatement> for ir::SourceStatement {
    fn desugar(node: &ast::ExpressionStatement, context: &mut LowerContext) -> LowerResult<Self> {
        let exports: Box<[ir::ExportAttribute]> = Box::desugar(&node.attr, context)?;
        // Without a name, the file cannot be named after the statement.
        exports
            .iter()
            .filter(|export| export.file.is_none())
            .for_each(|export| {
                context.push_err(LowerError::ExportWithoutFile {
                    src_ref: export.src_ref,
                })
            });

        Ok(Self {
            exports,
            attr: ir::ModelAttributes::desugar(&node.attr, context)?,
            src_ref: context.span_to_src_ref(&node.span),
            name: None,
            ty: ir::Type::default(),
            expression: ir::WorkbenchExpression::desugar(&node.expr, context)?,
        })
    }
}

impl Desugar<ast::LocalAssignment> for ir::SourceStatement {
    fn desugar(node: &ast::LocalAssignment, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            exports: Box::desugar(&node.attr, context)?,
            attr: ir::ModelAttributes::desugar(&node.attr, context)?,
            src_ref: context.span_to_src_ref(&node.span),
            name: Some(ir::Identifier::desugar(&node.id, context)?),
            ty: ir::Type::desugar(&node.ty, context)?,
            expression: ir::WorkbenchExpression::desugar(node.expr.as_ref(), context)?,
        })
    }
}

impl Desugar<ast::ExpressionStatement> for Option<ir::SourceStatement> {
    fn desugar(node: &ast::ExpressionStatement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Some(ir::SourceStatement::desugar(node, context)?))
    }
}

impl Desugar<ast::Statement> for Option<ir::SourceStatement> {
    fn desugar(stmt: &ast::Statement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match stmt {
            ast::Statement::LocalAssignment(local_assignment) => {
                Some(ir::SourceStatement::desugar(local_assignment, context)?)
            }
            ast::Statement::Expression(expression_statement) => {
                Some(ir::SourceStatement::desugar(expression_statement, context)?)
            }
            _ => None,
        })
    }
}

impl Desugar<ast::Source> for ir::desugared::Source {
    fn desugar(node: &ast::Source, context: &mut LowerContext) -> LowerResult<Self> {
        let statements = &node.statements;

        Ok(Self {
            meta: ir::Meta {
                name: context.source.file_module_name().map(Identifier::from),
                vis: Visibility::Public,
                src_ref: context.span_to_src_ref(&node.span),
                keyword_src_ref: SrcRef::none(),
            },
            attr: ir::Attributes::desugar(statements, context)?,
            items: ir::desugared::SourceItems::desugar(statements, context)?,
            statements: Box::desugar(statements, context)?,
        })
    }
}
