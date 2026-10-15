// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    CastInto, Desugar, LowerContext, LowerError, LowerResult,
    desugar::{extract_statements_with_tail, for_each_statement},
    ir,
};

use microcad_lang_base::{Identifier, PushIssue, SpanToSrcRef, SrcRef, SrcReferrer};
use microcad_lang_parse::ast;

impl Desugar<ast::def::Function> for ir::Attributes {
    fn desugar(node: &ast::def::Function, context: &mut LowerContext) -> LowerResult<Self> {
        crate::desugar::attribute::outer_with_doc(&node.doc, &node.attr, context)
    }
}

impl Desugar<ast::def::Function> for ir::FunctionSignature {
    fn desugar(node: &ast::def::Function, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            src_ref: context.span_to_src_ref(&node.span),
            parameters: ir::ParameterList::desugar(&node.parameters, context)?,
            return_type: match &node.return_type {
                Some(ty) => Some(ir::Type::desugar(ty, context)?),
                None => None,
            },
        })
    }
}

impl Desugar<ast::Body> for ir::Scope {
    fn desugar(node: &ast::Body, context: &mut LowerContext) -> LowerResult<Self> {
        let statements = &node.statements;
        for_each_statement(statements, context, |stmt, context| {
            let src_ref = context.span_to_src_ref(&stmt.span());
            use ast::Statement::*;
            match stmt {
                FileModule(_) | Const(_) | Use(_) | InlineModule(_) | Init(_) | Workbench(_)
                | Function(_) | Property(_) | InnerAttribute(_) | InnerDocComment(_) | Error(_) => {
                    context.push_err(LowerError::StatementNotAllowed { src_ref });
                }
                _ => {}
            }
            Ok(())
        })?;

        let statements: Box<[ir::FunctionStatement]> = Box::desugar(statements, context)?;

        Ok(Self {
            statements,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::FormatString> for ir::FunctionExpression {
    fn desugar(node: &ast::FormatString, context: &mut LowerContext) -> LowerResult<Self> {
        // Lowering format string must only contain constant expression (without `{}` bodies).
        // Hence, we need to `cast_into` the resulting `ir::ConstantExpression` into `ir::FunctionExpression`
        Ok(ir::Call::desugar(node, context)?.cast_into().into())
    }
}

impl Desugar<ast::Call> for ir::FunctionExpression {
    fn desugar(node: &ast::Call, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(ir::Call::desugar(node, context)?.into())
    }
}

impl Desugar<ast::Literal> for ir::FunctionExpression {
    fn desugar(node: &ast::Literal, context: &mut LowerContext) -> LowerResult<Self> {
        // Lower the literal expression `1m` -> `Length::mm(1000)` (includes unit conversion)
        Ok(ir::ConstantValue::desugar(node, context)?.into())
    }
}

impl Desugar<ast::BinaryOperation> for ir::FunctionExpression {
    fn desugar(node: &ast::BinaryOperation, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(ir::Call::desugar(node, context)?.into())
    }
}

impl Desugar<ast::Expression> for ir::FunctionExpression {
    fn desugar(node: &ast::Expression, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match node {
            // Remove parenthesis ()
            ast::Expression::Bracketed(expr, _) => Self::desugar(expr.as_ref(), context)?,
            // Lower call expression
            ast::Expression::Call(expr) => Self::desugar(expr, context)?,
            ast::Expression::Literal(expr) => Self::desugar(expr, context)?,
            ast::Expression::BinaryOperation(binop) => Self::desugar(binop, context)?,
            ast::Expression::String(s) => Self::desugar(s, context)?,
            // `(1, 2)` -> `__mu::core::tuple(1, 2)`
            ast::Expression::Tuple(t) => ir::Call::desugar(t, context)?.into(),
            ast::Expression::Range(a) => Self::desugar(a, context)?,
            ast::Expression::List(a) => Self::desugar(a, context)?,
            ast::Expression::SymbolPath(n) => ir::Path::desugar(n, context)?.into(),
            ast::Expression::UnaryOperation(unop) => ir::Call::desugar(unop, context)?.into(),
            ast::Expression::Marker(_) => {
                panic!("Marker statement not allowed")
            }
            ast::Expression::Body(body) => Self::Scope(ir::Scope::desugar(body, context)?),
            ast::Expression::ElementAccess(access) => Self::desugar(access, context)?,
            ast::Expression::If(if_expr) => Self::If(ir::If::desugar(if_expr, context)?),
            ast::Expression::Error(_) => todo!(),
        })
    }
}

impl Desugar<Option<ast::Expression>> for Option<ir::FunctionExpression> {
    fn desugar(node: &Option<ast::Expression>, context: &mut LowerContext) -> LowerResult<Self> {
        node.as_ref()
            .map(|expr| ir::FunctionExpression::desugar(expr, context))
            .transpose()
    }
}

impl Desugar<ast::Return> for ir::ReturnStatement {
    fn desugar(node: &ast::Return, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            expr: Option::<ir::FunctionExpression>::desugar(&node.expr, context)?,
            keyword_src_ref: context.span_to_src_ref(&node.keyword_span),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::Statement> for Option<ir::FunctionStatement> {
    fn desugar(stmt: &ast::Statement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match stmt {
            ast::Statement::Return(ret) => Some(ir::FunctionStatement::Return(
                ir::ReturnStatement::desugar(ret, context)?,
            )),
            ast::Statement::LocalAssignment(local_assignment) => {
                Some(ir::FunctionStatement::Local(ir::LocalAssignment::<
                    ir::FunctionExpression,
                >::desugar(
                    local_assignment, context
                )?))
            }
            ast::Statement::Expression(expression_statement) => {
                let src_ref = context.span_to_src_ref(&expression_statement.span);
                use ast::Expression::*;
                match &expression_statement.expr {
                    Literal(_)
                    | Bracketed(_, _)
                    | Tuple(_)
                    | Range(_)
                    | List(_)
                    | String(_)
                    | SymbolPath(_)
                    | BinaryOperation(_)
                    | UnaryOperation(_)
                    | ElementAccess(_) => {
                        context.push_err(LowerError::FunctionStatementIgnored(src_ref));
                        None
                    }
                    ast::Expression::Marker(_) | ast::Expression::Error(_) => {
                        context.push_err(LowerError::StatementNotAllowed { src_ref });
                        None
                    }
                    ast::Expression::Call(call) => Some(ir::Call::desugar(call, context)?.into()),
                    ast::Expression::Body(body) => Some(ir::Scope::desugar(body, context)?.into()),
                    ast::Expression::If(if_) => Some(ir::If::desugar(if_, context)?.into()),
                }
            }
            _ => None,
        })
    }
}

impl Desugar<ast::StatementList> for Box<[ir::FunctionStatement]> {
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        let statements = extract_statements_with_tail(
            node,
            context,
            Option::<ir::FunctionStatement>::desugar,
            // Lower Tail expression to Return statements.
            |tail, context| {
                Ok(Some(ir::FunctionStatement::Tail(
                    ir::FunctionExpression::desugar(&tail.expr, context)?,
                )))
            },
        )?;

        let mut return_src_ref = SrcRef::none();

        for stmt in statements.iter() {
            let src_ref = stmt.src_ref();
            if return_src_ref.is_some() {
                // We've already hit a return, so everything after it is unreachable dead code.
                context.push_err(LowerError::Unreachable {
                    src_ref,
                    last_ref: return_src_ref,
                });
            } else if let ir::FunctionStatement::Return(ret) = stmt {
                // Found the first return statement!
                return_src_ref = ret.src_ref;
            }
        }

        Ok(statements)
    }
}

impl Desugar<ast::StatementList> for ir::desugared::FunctionItems {
    fn desugar(statements: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        for_each_statement(statements, context, |stmt, context| {
            let src_ref = context.span_to_src_ref(&stmt.span());
            use ast::Statement::*;
            match stmt {
                Init(_) | Workbench(_) | InlineModule(_) | FileModule(_) | Property(_)
                | Error(_) => context.push_err(LowerError::StatementNotAllowed { src_ref }),
                _ => {}
            }
            Ok(())
        })?;

        Ok(Self {
            aliases: ir::desugared::Aliases::desugar(statements, context)?,
            constants: Box::desugar(statements, context)?,
        })
    }
}

impl Desugar<ast::def::Function> for ir::desugared::Function {
    fn desugar(node: &ast::def::Function, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            meta: ir::Meta {
                name: Some(Identifier::desugar(&node.id, context)?),
                src_ref: context.span_to_src_ref(&node.span),
                keyword_src_ref: context.span_to_src_ref(&node.keyword_span),
                vis: ir::Visibility::desugar(&node.vis, context)?,
            },
            attr: ir::Attributes::desugar(node, context)?
                .extend(ir::Attributes::desugar(&node.body.statements, context)?),
            signature: ir::FunctionSignature::desugar(node, context)?,
            items: ir::desugared::FunctionItems::desugar(&node.body.statements, context)?,
            statements: Box::desugar(&node.body.statements, context)?,
        })
    }
}

impl Desugar<ast::Statement> for Option<ir::desugared::Function> {
    fn desugar(node: &ast::Statement, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match node {
            ast::Statement::Function(f) => Some(ir::desugared::Function::desugar(f, context)?),
            _ => None,
        })
    }
}

impl Desugar<ast::ExpressionStatement> for Option<ir::desugared::Function> {
    fn desugar(_: &ast::ExpressionStatement, _: &mut LowerContext) -> LowerResult<Self> {
        Ok(None)
    }
}
