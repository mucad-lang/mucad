// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerError, LowerResult, desugar::check_for_duplicates, ir};

use mucad_lang_base::{PushIssue, SpanToSrcRef};
use mucad_lang_parse::ast;

/// Trait to lower generic attributes into item specific attributes.
pub trait LowerAttributes: Sized + Default {
    fn lower_attributes(attr: ir::Attributes, context: &mut LowerContext) -> LowerResult<Self>;
}

impl LowerAttributes for ir::ParameterAttributes {
    fn lower_attributes(attr: ir::Attributes, context: &mut LowerContext) -> LowerResult<Self> {
        attr.commands.iter().for_each(|attr| {
            context.push_err(LowerError::UnsupportedCommandAttribute {
                path: attr.path.clone(),
                src_ref: attr.src_ref,
            })
        });
        attr.kv_exprs.iter().for_each(|attr| {
            context.push_err(LowerError::UnsupportedKeyValueAttribute {
                key: attr.name.clone(),
                src_ref: attr.src_ref,
            })
        });
        attr.tags.iter().for_each(|attr| {
            context.push_err(LowerError::UnsupportedTagAttribute {
                tag: attr.name.clone(),
            })
        });

        Ok(Self { doc: attr.doc })
    }
}

impl Desugar<ast::Parameter> for ir::Parameter {
    fn desugar(node: &ast::Parameter, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            attr: ir::ParameterAttributes::lower_attributes(
                crate::desugar::attribute::outer_with_doc(&node.doc, &node.attr, context)?,
                context,
            )?,
            src_ref: context.span_to_src_ref(&node.span),
            id: ir::Identifier::desugar(&node.id, context)?,
            ty: ir::Type::desugar(&node.ty, context)?,
            default_value: node
                .default
                .as_ref()
                .map(|def| ir::ConstantExpression::desugar(def, context))
                .transpose()?,
        })
    }
}

impl Desugar<ast::ParameterList> for ir::ParameterList {
    fn desugar(node: &ast::ParameterList, context: &mut LowerContext) -> LowerResult<Self> {
        let mut parameters = Vec::new();

        for param in &node.parameters {
            parameters.push(ir::Parameter::desugar(param, context)?);
        }

        Ok(ir::ParameterList {
            parameters: check_for_duplicates(parameters, context)?,
            src_ref: context.span_to_src_ref(&node.span),
        })
    }
}
