// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerError, LowerResult, desugar::DesugarExpr, ir};

mod call;
mod literal;

use microcad_builtin::{__mu, BuiltinError};
use microcad_lang_base::{Identifier, PushIssue, SpanToSrcRef};
use microcad_lang_parse::ast;
use microcad_lang_types::{Scalar, Value};

impl Desugar<ast::Identifier> for ir::Marker {
    fn desugar(node: &ast::Identifier, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            id: Identifier::desugar(node, context)?,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl<Expr: ir::ExprSpec> Desugar<ast::If> for ir::If<Expr>
where
    Expr: Desugar<ast::Expression>,
    Expr::Body: Desugar<ast::Body>,
{
    fn desugar(node: &ast::If, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(ir::If {
            if_ref: context.span_to_src_ref(&node.if_span),
            cond: Box::new(Expr::desugar(node.condition.as_ref(), context)?),
            body: Expr::Body::desugar(&node.body, context)?.into(),
            next_if: node
                .next_if
                .as_ref()
                .map(|next| ir::If::desugar(next, context))
                .transpose()?
                .map(Box::new),
            else_ref: node
                .else_span
                .as_ref()
                .map(|span| context.span_to_src_ref(span)),
            body_else: node
                .else_body
                .as_ref()
                .map(|body| Expr::Body::desugar(body, context))
                .transpose()?
                .map(Box::new),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::SymbolPath> for ir::Path {
    fn desugar(node: &ast::SymbolPath, context: &mut LowerContext) -> LowerResult<Self> {
        let path = ir::UnresolvedPath {
            is_absolute: node.prefix.is_some(),
            parts: node
                .parts
                .iter()
                .map(|ident| ir::Identifier::desugar(ident, context))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            src_ref: context.span_to_src_ref(&node.span),
        };

        if let Some(id) = path.builtin_id() {
            if context.builtins.get(id).is_some() {
                return Ok(id.into());
            } else {
                context.push_err(BuiltinError::NoBuiltin {
                    full_name: path.to_string(),
                    id,
                });
            }
        }

        Ok(Self::Unresolved(path))
    }
}

impl<Expr: DesugarExpr> Desugar<ast::RangeExpression> for Expr
where
    Expr: From<ir::Call<Expr>> + From<ir::ConstantValue>,
{
    fn desugar(a: &ast::RangeExpression, context: &mut LowerContext) -> LowerResult<Self> {
        let unit = ir::Unit::desugar(&a.unit, context)?;
        let range = Expr::from(ir::Call {
            path: __mu!(core::range).into(),
            args: ir::ArgumentList::from_iter([
                ir::Argument::named("start", Expr::desugar(&a.start.expr, context)?),
                ir::Argument::named("end", Expr::desugar(&a.end.expr, context)?),
            ]),
            src_ref: context.span_to_src_ref(&a.span),
        });

        Ok(if unit.is_none() {
            range
        } else {
            Expr::from(ir::Call {
                path: __mu!(core::mul).into(),
                args: ir::ArgumentList::from_iter([
                    ir::Argument::named("lhs", range),
                    ir::Argument::named(
                        "rhs",
                        Expr::from(ir::ConstantValue::from(
                            (Value::from(Scalar::from_num(1.0)) * unit)?,
                        )),
                    ),
                ]),
                src_ref: context.span_to_src_ref(&a.span),
            })
        })
    }
}

impl<Expr: DesugarExpr> Desugar<ast::ListExpression> for Expr
where
    Expr: From<ir::Call<Expr>> + From<ir::ConstantValue>,
{
    fn desugar(a: &ast::ListExpression, context: &mut LowerContext) -> LowerResult<Self> {
        let unit = ir::Unit::desugar(&a.unit, context)?;

        let args = a
            .items
            .iter()
            .map(|item| Expr::desugar(&item.expr, context))
            .collect::<Result<Vec<Expr>, _>>()?;

        let list = Expr::from(ir::Call {
            path: __mu!(core::list).into(),
            args: ir::ArgumentList::from_iter(args.into_iter().map(ir::Argument::Unnamed)),
            src_ref: context.span_to_src_ref(&a.span),
        });

        Ok(if unit.is_none() {
            list
        } else {
            Expr::from(ir::Call {
                path: __mu!(core::mul).into(),
                args: ir::ArgumentList::from_iter([
                    ir::Argument::named("lhs", list),
                    ir::Argument::named(
                        "rhs",
                        Expr::from(ir::ConstantValue::from(
                            (Value::from(Scalar::from_num(1.0)) * unit)?,
                        )),
                    ),
                ]),
                src_ref: context.span_to_src_ref(&a.span),
            })
        })
    }
}

impl<Expr: DesugarExpr> Desugar<ast::TupleExpression> for ir::Call<Expr>
where
    Expr: From<ir::Call<Expr>> + From<ir::ConstantValue>,
{
    fn desugar(node: &ast::TupleExpression, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            path: __mu!(core::tuple).into(),
            args: ir::ArgumentList::desugar(&node.values, context)?,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::Expression> for ir::ConstantExpression {
    fn desugar(node: &ast::Expression, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match node {
            ast::Expression::Bracketed(expr, _) => Self::desugar(expr.as_ref(), context)?,
            ast::Expression::Literal(expr) => ir::ConstantValue::desugar(expr, context)?.into(),
            ast::Expression::String(s) => ir::Call::desugar(s, context)?.into(),
            ast::Expression::Tuple(t) => ir::Call::desugar(t, context)?.into(),
            ast::Expression::Range(a) => Self::desugar(a, context)?,
            ast::Expression::List(a) => Self::desugar(a, context)?,
            ast::Expression::SymbolPath(n) => Self::Path(ir::Path::desugar(n, context)?),
            ast::Expression::BinaryOperation(binop) => {
                Self::Call(ir::Call::desugar(binop, context)?)
            }
            ast::Expression::UnaryOperation(unop) => Self::Call(ir::Call::desugar(unop, context)?),
            expr => {
                context.push_err(LowerError::InvalidConstantExpression {
                    src_ref: context.span_to_src_ref(&expr.span()),
                });
                Self::Invalid
            }
        })
    }
}
