// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Lowering the AST.

mod attribute;
mod constant;
mod expression;
mod function;
mod module;
mod parameter;
mod source;
mod r#type;
mod workbench;

#[cfg(test)]
mod tests;

use mucad_builtin::BuiltinError;
use mucad_lang_base::{Identifiable, PushIssue, Refer, SpanToSrcRef, Spanned, SrcReferrer};
use mucad_lang_parse::ast;

use crate::{Desugar, LowerContext, LowerError, LowerResult, ir};

pub trait DesugarExpr: ir::ExprSpec + Desugar<ast::Expression> {}

impl DesugarExpr for ir::FunctionExpression {}
impl DesugarExpr for ir::WorkbenchExpression {}
impl DesugarExpr for ir::ConstantExpression {}

/// Extracts and maps specific variants out of a statement collection tuple list.
///
/// Does not check if the statements are actually valid in this context.
pub fn extract_statements_with_tail<F, G, T>(
    statements: &ast::StatementList,
    context: &mut LowerContext,
    mut extractor: F,
    mut tail_extractor: G,
) -> LowerResult<Box<[T]>>
where
    F: FnMut(&ast::Statement, &mut LowerContext) -> LowerResult<Option<T>>,
    G: FnMut(&ast::ExpressionStatement, &mut LowerContext) -> LowerResult<Option<T>>,
{
    let mut mapped = Vec::new();
    statements
        .statements
        .iter()
        .map(|(stmt, _)| stmt)
        .try_for_each(|stmt| -> LowerResult<()> {
            mapped.extend(extractor(stmt, context)?);
            Ok(())
        })?;

    if let Some(tail) = &statements.tail {
        mapped.extend(tail_extractor(tail, context)?);
    }
    Ok(mapped.into_boxed_slice())
}

/// Extracts and maps specific variants out of a statement collection tuple list.
///
/// Does not check if the statements are actually valid in this context.
pub fn extract_statements<F, T>(
    statements: &ast::StatementList,
    mut extractor: F,
) -> LowerResult<Box<[T]>>
where
    F: FnMut(&ast::Statement) -> LowerResult<Option<T>>,
{
    let mut mapped = Vec::new();
    statements
        .statements
        .iter()
        .map(|(stmt, _)| stmt)
        .try_for_each(|stmt| -> LowerResult<()> {
            if let Some(m) = extractor(stmt)? {
                mapped.push(m);
            }
            Ok(())
        })?;

    Ok(mapped.into_boxed_slice())
}

pub fn for_each_statement<F>(
    statements: &ast::StatementList,
    context: &mut LowerContext,
    mut check: F,
) -> LowerResult<()>
where
    F: FnMut(&ast::Statement, &mut LowerContext) -> LowerResult<()>,
{
    statements
        .statements
        .iter()
        .map(|(stmt, _)| stmt)
        .try_for_each(|stmt| check(stmt, context))
}

/// Sort by name and check for duplicates
pub fn check_for_duplicates<T>(named: Vec<T>, context: &mut LowerContext) -> LowerResult<Box<[T]>>
where
    T: Identifiable + SrcReferrer + Clone,
{
    let mut sorted = named.clone();
    sorted.sort_by_key(|lhs| lhs.id());
    sorted
        .windows(2)
        .filter_map(|pair| {
            if pair[0].id() == pair[1].id() {
                Some((&pair[0], &pair[1]))
            } else {
                None
            }
        })
        .try_for_each(|(prev_arg, arg)| -> LowerResult<()> {
            context.push_err(LowerError::DuplicateArgument {
                id: arg.id().clone(),
                previous: prev_arg.id().clone(),
            });
            Ok(())
        })?;

    Ok(named.into_boxed_slice())
}

impl<T> Desugar<ast::StatementList> for Box<[T]>
where
    Option<T>: Desugar<ast::Statement>,
    Option<T>: Desugar<ast::ExpressionStatement>,
{
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        extract_statements_with_tail(node, context, Option::desugar, Option::desugar)
    }
}

impl Desugar<Option<Spanned<ast::def::Visibility>>> for ir::Visibility {
    fn desugar(
        node: &Option<Spanned<ast::def::Visibility>>,
        _context: &mut LowerContext,
    ) -> LowerResult<Self> {
        Ok(match node.as_ref().map(|v| &v.value) {
            Some(ast::def::Visibility::Public) => Self::Public,
            None => Self::Private,
        })
    }
}

impl Desugar<ast::Identifier> for ir::Identifier {
    fn desugar(node: &ast::Identifier, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self(Refer::new(
            node.name.clone(),
            context.span_to_src_ref(&node.span),
        )))
    }
}

impl Desugar<ast::def::UseName> for ir::Path {
    fn desugar(node: &ast::def::UseName, context: &mut LowerContext) -> LowerResult<Self> {
        let path = ir::UnresolvedPath {
            is_absolute: node.prefix.is_some(),
            parts: node
                .parts
                .iter()
                .filter_map(|part| match part {
                    ast::def::UseStatementPart::Identifier(ident) => {
                        Some(ir::Identifier::desugar(ident, context))
                    }
                    ast::def::UseStatementPart::Glob(_) => None,
                    ast::def::UseStatementPart::Error(_) => None,
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            src_ref: context.span_to_src_ref(&node.span),
        };

        if let Some(id) = path.builtin_id() {
            if context.builtins.get(id).is_none() {
                context.push_err(BuiltinError::NoBuiltin {
                    full_name: path.to_string(),
                    id,
                });
            } else {
                return Ok(id.into());
            }
        }

        Ok(path.into())
    }
}

impl<Expr> Desugar<ast::LocalAssignment> for ir::LocalAssignment<Expr>
where
    Expr: Desugar<ast::Expression>,
{
    fn desugar(node: &ast::LocalAssignment, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            id: ir::Identifier::desugar(&node.id, context)?,
            ty: ir::Type::desugar(&node.ty, context)?,
            expression: Expr::desugar(node.expr.as_ref(), context)?,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::StatementList> for ir::desugared::Aliases {
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            explicit_aliases: extract_statements(node, |stmt| match stmt {
                ast::Statement::Use(use_statement) => match use_statement.name.parts.last() {
                    Some(ast::def::UseStatementPart::Identifier(id)) => {
                        Ok(Some(ir::desugared::Alias {
                            meta: ir::Meta {
                                keyword_src_ref: context
                                    .span_to_src_ref(&use_statement.keyword_span),
                                vis: ir::Visibility::desugar(&use_statement.vis, context)?,
                                name: Some(ir::Identifier::desugar(
                                    match &use_statement.use_as {
                                        // Use id `C` from `as C`
                                        Some(id) => id,
                                        // Use id `Circle` from last part of path `std::geo2d::Circle`
                                        None => id,
                                    },
                                    context,
                                )?),
                                src_ref: context.span_to_src_ref(&use_statement.span),
                            },
                            attr: ir::Attributes::desugar(&use_statement.attr, context)?,
                            path: ir::Path::desugar(&use_statement.name, context)?,
                        }))
                    }
                    None => unreachable!(),
                    Some(_) => Ok(None),
                },
                _ => Ok(None),
            })?,
            wildcards: extract_statements(node, |stmt| match stmt {
                ast::Statement::Use(use_statement) => match use_statement.name.parts.last() {
                    Some(ast::def::UseStatementPart::Glob(_)) => {
                        Ok(Some(ir::desugared::Wildcard {
                            meta: ir::Meta {
                                name: None,
                                keyword_src_ref: context
                                    .span_to_src_ref(&use_statement.keyword_span),
                                vis: ir::Visibility::desugar(&use_statement.vis, context)?,
                                src_ref: context.span_to_src_ref(&use_statement.span),
                            },
                            attr: ir::Attributes::desugar(&use_statement.attr, context)?,
                            path: ir::Path::desugar(&use_statement.name, context)?,
                        }))
                    }
                    None => unreachable!(),
                    Some(_) => Ok(None),
                },
                _ => Ok(None),
            })?,
        })
    }
}
