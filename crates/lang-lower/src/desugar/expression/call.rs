// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerError, LowerResult, desugar::DesugarExpr, ir};

use microcad_builtin::__mu;
use microcad_lang_base::{Identifier, PushIssue, SpanToSrcRef, SrcRef};
use microcad_lang_parse::ast;

impl<Expr: DesugarExpr> Desugar<ast::Call> for ir::Call<Expr> {
    fn desugar(node: &ast::Call, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(ir::Call {
            src_ref: context.span_to_src_ref(&node.span),
            path: ir::Path::desugar(&node.path, context)?,
            args: ir::ArgumentList::desugar(&node.arguments, context)?,
        })
    }
}

impl<Expr: DesugarExpr> Desugar<Vec<ast::TupleItem>> for ir::ArgumentList<Expr> {
    fn desugar(node: &Vec<ast::TupleItem>, context: &mut LowerContext) -> LowerResult<Self> {
        let mut args = Vec::new();
        let mut names: microcad_lang_base::HashSet<Identifier> =
            microcad_lang_base::HashSet::default();

        node.iter().try_for_each(|arg| -> LowerResult<()> {
            let expr = Expr::desugar(&arg.expr, context)?;
            let src_ref = context.span_to_src_ref(&arg.span);

            let arg = match &arg.id {
                Some(name) => ir::Argument::Named {
                    name: ir::Identifier::desugar(name, context)?,
                    expr,
                    src_ref,
                },
                None => ir::Argument::from(expr),
            };

            if let Some(name) = arg.name() {
                if names.contains(name) {
                    context.push_err(LowerError::DuplicateArgument {
                        id: name.clone(),
                        previous: names.get(name).unwrap().clone(),
                    });
                    return Ok(());
                }
                names.extend(arg.name().cloned());
            }

            args.push(arg);

            Ok(())
        })?;

        Ok(Self {
            src_ref: SrcRef::none(),
            args: args.into_boxed_slice(),
        })
    }
}

impl<Expr: DesugarExpr> Desugar<ast::ArgumentList> for ir::ArgumentList<Expr> {
    fn desugar(node: &ast::ArgumentList, context: &mut LowerContext) -> LowerResult<Self> {
        let mut args = Vec::new();

        node.arguments
            .iter()
            .try_for_each(|arg| -> LowerResult<()> {
                match arg.name() {
                    Some(name) => args.push(ir::Argument::Named {
                        name: ir::Identifier::desugar(name, context)?,
                        expr: Expr::desugar(arg.value(), context)?,
                        src_ref: context.span_to_src_ref(arg.span()),
                    }),
                    None => {
                        let expr = Expr::desugar(arg.value(), context)?;
                        args.push(ir::Argument::from(expr));
                    }
                }
                Ok(())
            })?;

        Ok(Self {
            src_ref: context.span_to_src_ref(&node.span),
            args: args.into_boxed_slice(),
        })
    }
}

impl<Expr: DesugarExpr> Desugar<ast::UnaryOperation> for ir::Call<Expr> {
    fn desugar(node: &ast::UnaryOperation, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            path: match node.op.value {
                ast::UnaryOperator::Minus => __mu!(core::neg),
                ast::UnaryOperator::Plus => __mu!(core::plus),
                ast::UnaryOperator::Not => __mu!(core::not),
            }
            .into(),
            args: ir::ArgumentList::from_iter([Expr::desugar(&node.rhs, context)?]),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl<Expr: DesugarExpr> Desugar<ast::BinaryOperation> for ir::Call<Expr> {
    fn desugar(node: &ast::BinaryOperation, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            path: match node.op.value {
                ast::BinaryOperator::Add => __mu!(core::add),
                ast::BinaryOperator::Subtract => __mu!(core::sub),
                ast::BinaryOperator::Multiply => __mu!(core::mul),
                ast::BinaryOperator::Divide => __mu!(core::div),
                ast::BinaryOperator::Union => __mu!(core::union),
                ast::BinaryOperator::Intersect => __mu!(core::intersect),
                ast::BinaryOperator::GreaterThan => __mu!(core::gt),
                ast::BinaryOperator::LessThan => __mu!(core::lt),
                ast::BinaryOperator::GreaterEqual => __mu!(core::ge),
                ast::BinaryOperator::LessEqual => __mu!(core::le),
                ast::BinaryOperator::Equal => __mu!(core::eq),
                ast::BinaryOperator::Near => __mu!(core::near),
                ast::BinaryOperator::NotEqual => __mu!(core::not_equal),
                ast::BinaryOperator::And => __mu!(core::and),
                ast::BinaryOperator::Or => __mu!(core::or),
                ast::BinaryOperator::Xor | ast::BinaryOperator::PowerXor => __mu!(core::xor),
            }
            .into(),
            args: ir::ArgumentList::from_iter([
                ir::Argument::Named {
                    name: Identifier::no_ref("lhs"),
                    expr: Expr::desugar(&node.lhs, context)?,
                    src_ref: context.span_to_src_ref(&node.lhs.span()),
                },
                ir::Argument::Named {
                    name: Identifier::no_ref("rhs"),
                    expr: Expr::desugar(&node.rhs, context)?,
                    src_ref: context.span_to_src_ref(&node.rhs.span()),
                },
            ]),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

pub fn lower_spec(
    expr: ir::ConstantExpression,
    spec: &ast::StringFormatSpecification,
    context: &mut LowerContext,
) -> LowerResult<ir::Call<ir::ConstantExpression>> {
    let width = spec
        .width
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .copied()
        .map(|w| ir::ConstantValue::new(w as i64).into())
        .unwrap_or(ir::ConstantExpression::Invalid);

    let precision = spec
        .precision
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .copied()
        .map(|p| ir::ConstantValue::new(p as i64).into())
        .unwrap_or(ir::ConstantExpression::Invalid);

    Ok(ir::Call {
        path: __mu!(core::format_spec).into(),
        args: ir::ArgumentList::from_iter([expr, width, precision]),
        src_ref: context.span_to_src_ref(&spec.span),
    })
}

impl Desugar<ast::FormatString> for ir::Call<ir::ConstantExpression> {
    fn desugar(node: &ast::FormatString, context: &mut LowerContext) -> LowerResult<Self> {
        let mut args_vec = Vec::new();
        let mut pending_str = String::new();

        for part in &node.parts {
            match part {
                ast::StringPart::Char(c) => {
                    pending_str.push(c.character);
                }
                ast::StringPart::Content(lit) => {
                    pending_str.push_str(&lit.content);
                }
                ast::StringPart::Expression(expr_part) => {
                    // Flush accumulated string literal first
                    if !pending_str.is_empty() {
                        args_vec.push(ir::ConstantExpression::from(ir::ConstantValue::new(
                            std::mem::take(&mut pending_str),
                        )));
                    }

                    let lowered_expr =
                        ir::ConstantExpression::desugar(expr_part.expr.as_ref(), context)?;

                    if expr_part.specification.is_some() {
                        args_vec.push(
                            lower_spec(lowered_expr, &expr_part.specification, context)?.into(),
                        );
                    } else {
                        args_vec.push(lowered_expr);
                    }
                }
            }
        }

        // Flush remaining trailing string content
        if !pending_str.is_empty() {
            args_vec.push(ir::ConstantExpression::from(ir::ConstantValue::new(
                pending_str,
            )));
        }

        Ok(Self {
            path: __mu!(core::format).into(),
            args: ir::ArgumentList::from_iter(args_vec),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}
