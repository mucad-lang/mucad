// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::desugar::{extract_statements, for_each_statement};
use crate::ir::source::ExportAttribute;
use crate::{Desugar, LowerContext, LowerError, LowerResult, ir};

use mucad_lang_base::{PushIssue, SingleIdentifier, SpanToSrcRef, SrcReferrer};
use mucad_lang_parse::ast;

/// Helper function to get outer attributes
pub fn outer_with_doc(
    doc: &ast::DocBlock,
    attr: &ast::Attributes,
    context: &mut LowerContext,
) -> LowerResult<ir::Attributes> {
    let mut attr = ir::Attributes::desugar(attr, context)?;
    attr.doc = ir::DocBlock::desugar(doc, context)?;
    Ok(attr)
}

fn extract_attributes<'a, T, F>(
    i: impl Iterator<Item = &'a ast::Attribute>,
    mut f: F,
) -> LowerResult<Box<[T]>>
where
    F: FnMut(&ast::AttributeCommand) -> LowerResult<Option<T>>,
{
    let mut items = Vec::new();
    i.flat_map(|attr| attr.commands.iter())
        .try_for_each(|cmd| -> LowerResult<()> {
            if let Some(item) = f(cmd)? {
                items.push(item);
            }
            Ok(())
        })?;

    Ok(items.into_boxed_slice())
}

impl Desugar<ast::Attributes> for ir::Attributes {
    fn desugar(node: &ast::Attributes, context: &mut LowerContext) -> LowerResult<Self> {
        // Generate outer attributes without doc
        Ok(ir::Attributes {
            doc: ir::DocBlock::default(),
            kv_exprs: Box::<[ir::KvExpr]>::desugar(node, context)?,
            commands: Box::<[ir::Command]>::desugar(node, context)?,
            tags: Box::<[ir::Tag]>::desugar(node, context)?,
        })
    }
}

impl Desugar<ast::DocBlock> for ir::DocBlock {
    fn desugar(node: &ast::DocBlock, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            content: node
                .lines
                .iter()
                .filter_map(|s| s.strip_prefix("/// ").or(s.strip_prefix("///")))
                .map(|s| s.trim_end().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::StatementList> for ir::DocBlock {
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        // This does not check if statements are allowed in this context
        Ok(Self {
            content: extract_statements(node, |stmt| {
                Ok(match stmt {
                    ast::Statement::InnerDocComment(inner_doc_comment) => {
                        let s = inner_doc_comment.line.clone();
                        let s = s.strip_prefix("//! ").or(s.strip_prefix("//!"));
                        s.map(|s| s.trim_end().to_string())
                    }
                    _ => None,
                })
            })?
            .join("\n"),
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::Attributes> for Box<[ir::KvExpr]> {
    fn desugar(node: &ast::Attributes, context: &mut LowerContext) -> LowerResult<Self> {
        extract_attributes(node.0.iter(), |cmd| -> LowerResult<_> {
            Ok(match cmd {
                ast::AttributeCommand::Assignment(local_assignment) => {
                    Some(ir::KvExpr::desugar(local_assignment, context)?)
                }
                _ => None,
            })
        })
    }
}

impl Desugar<ast::LocalAssignment> for ir::KvExpr {
    fn desugar(node: &ast::LocalAssignment, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(ir::KvExpr {
            name: ir::Path::from(ir::Identifier::desugar(&node.id, context)?),
            expr: ir::ConstantExpression::desugar(node.expr.as_ref(), context)?,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::StatementList> for Box<[ir::KvExpr]> {
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        extract_attributes(
            node.statements.iter().filter_map(|(stmt, _)| match stmt {
                ast::Statement::InnerAttribute(attribute) => Some(attribute),
                _ => None,
            }),
            |cmd| -> LowerResult<_> {
                Ok(match cmd {
                    ast::AttributeCommand::Assignment(local_assignment) => {
                        Some(ir::KvExpr::desugar(local_assignment, context)?)
                    }
                    _ => None,
                })
            },
        )
    }
}

impl Desugar<ast::Call> for ir::Command {
    fn desugar(node: &ast::Call, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            path: ir::Path::desugar(&node.path, context)?,
            argument_list: ir::ArgumentList::desugar(&node.arguments, context)?,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}

impl Desugar<ast::Attributes> for Box<[ir::Command]> {
    fn desugar(node: &ast::Attributes, context: &mut LowerContext) -> LowerResult<Self> {
        extract_attributes(node.0.iter(), |cmd| -> LowerResult<_> {
            Ok(match cmd {
                ast::AttributeCommand::Call(call) => Some(ir::Command::desugar(call, context)?),
                _ => None,
            })
        })
    }
}

impl Desugar<ast::StatementList> for Box<[ir::Command]> {
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        extract_attributes(
            node.statements.iter().filter_map(|(stmt, _)| match stmt {
                ast::Statement::InnerAttribute(attribute) => Some(attribute),
                _ => None,
            }),
            |cmd| -> LowerResult<_> {
                Ok(match cmd {
                    ast::AttributeCommand::Call(call) => Some(ir::Command::desugar(call, context)?),
                    _ => None,
                })
            },
        )
    }
}

impl Desugar<ast::Identifier> for ir::Tag {
    fn desugar(node: &ast::Identifier, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            name: ir::Identifier::desugar(node, context)?,
        })
    }
}

impl Desugar<ast::Attributes> for Box<[ir::Tag]> {
    fn desugar(node: &ast::Attributes, context: &mut LowerContext) -> LowerResult<Self> {
        extract_attributes(node.0.iter(), |cmd| -> LowerResult<_> {
            Ok(match cmd {
                ast::AttributeCommand::Ident(ident) => Some(ir::Tag::desugar(ident, context)?),
                _ => None,
            })
        })
    }
}

impl Desugar<ast::StatementList> for Box<[ir::Tag]> {
    fn desugar(node: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        extract_attributes(
            node.statements.iter().filter_map(|(stmt, _)| match stmt {
                ast::Statement::InnerAttribute(attribute) => Some(attribute),
                _ => None,
            }),
            |cmd| -> LowerResult<_> {
                Ok(match cmd {
                    ast::AttributeCommand::Ident(ident) => Some(ir::Tag::desugar(ident, context)?),
                    _ => None,
                })
            },
        )
    }
}

impl Desugar<ast::def::Workbench> for ir::Attributes {
    fn desugar(node: &ast::def::Workbench, context: &mut LowerContext) -> LowerResult<Self> {
        outer_with_doc(&node.doc, &node.attr, context)
    }
}

/// Lower inner attributes
impl Desugar<ast::StatementList> for ir::Attributes {
    fn desugar(statements: &ast::StatementList, context: &mut LowerContext) -> LowerResult<Self> {
        #[derive(PartialEq)]
        enum State {
            /// Try to read doc comments first
            InitDoc,
            /// Inner attribute must come after inner doc comments
            Attributes,
            /// Only statements afterwards.
            Statements,
        }

        // Check order of inner attribute statements.
        let mut state = State::InitDoc;
        for_each_statement(statements, context, |stmt, context| {
            let src_ref = context.span_to_src_ref(&stmt.span());
            match stmt {
                ast::Statement::InnerDocComment(_) => {
                    if state != State::InitDoc {
                        context.push_err(LowerError::InnerDocAfterInnerAttribute { src_ref });
                    }
                }
                ast::Statement::InnerAttribute(_) => {
                    if state == State::Statements {
                        context.push_err(LowerError::InnerAttributeAfterStatement { src_ref });
                    } else {
                        state = State::Attributes;
                    }
                }
                _ => state = State::Statements,
            }
            Ok(())
        })?;

        Ok(ir::Attributes {
            doc: ir::DocBlock::desugar(statements, context)?,
            kv_exprs: Box::<[ir::KvExpr]>::desugar(statements, context)?,
            commands: Box::<[ir::Command]>::desugar(statements, context)?,
            tags: Box::<[ir::Tag]>::desugar(statements, context)?,
        })
    }
}

impl ir::Attributes {
    /// The export attributes: `#[export]`, `#[export = "file.svg"]` or `#[export("file.svg")]`.
    pub(crate) fn fetch_exports(&self) -> LowerResult<Box<[ExportAttribute]>> {
        let is_export = |path: &ir::Path| {
            path.single_identifier()
                .is_some_and(|ident| ident.as_str() == "export")
        };

        let tags = self
            .tags
            .iter()
            .filter(|tag| tag.name.as_str() == "export")
            .map(|tag| ExportAttribute {
                file: None,
                src_ref: tag.src_ref(),
            });
        let kv_exprs = self
            .kv_exprs
            .iter()
            .filter(|kv_expr| is_export(&kv_expr.name))
            .map(|kv_expr| ExportAttribute {
                file: Some(kv_expr.expr.clone()),
                src_ref: kv_expr.src_ref,
            });
        let commands = self
            .commands
            .iter()
            .filter(|command| is_export(&command.path))
            .map(|command| ExportAttribute {
                file: command
                    .argument_list
                    .extract_named_arg_expr("file")
                    .or(command.argument_list.extract_first_unnamed_arg_expr())
                    .cloned(),
                src_ref: command.src_ref,
            });

        Ok(tags.chain(kv_exprs).chain(commands).collect())
    }
}
