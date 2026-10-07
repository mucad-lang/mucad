// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerResult, ir};

use microcad_lang_base::{Refer, SpanToSrcRef};
use microcad_lang_parse::ast;
use microcad_lang_types::{Type, ty};

impl Desugar<ast::Type> for Type {
    fn desugar(node: &ast::Type, context: &mut LowerContext) -> LowerResult<Self> {
        use std::str::FromStr;
        Ok(match node {
            ast::Type::Single(ty) => Type::from_str(ty.name.as_str())
                .map_err(|err| Refer::new(err, context.span_to_src_ref(&node.span())))?,
            ast::Type::List(ty) => Type::List(Box::new(Type::desugar(&ty.inner, context)?)),
            ast::Type::Tuple(ty) => Type::Tuple(Box::new(ty::TupleType::desugar(ty, context)?)),
        })
    }
}

impl Desugar<ast::Type> for ir::Type {
    fn desugar(node: &ast::Type, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            ty: Type::desugar(node, context)?,
            src_ref: context.span_to_src_ref(&node.span()),
        })
    }
}

impl Desugar<Option<ast::Type>> for ir::Type {
    fn desugar(node: &Option<ast::Type>, context: &mut LowerContext) -> LowerResult<Self> {
        match node {
            Some(ty) => ir::Type::desugar(ty, context),
            None => Ok(ir::Type::default()),
        }
    }
}

impl Desugar<ast::TupleType> for ir::TupleType {
    fn desugar(node: &ast::TupleType, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(Self {
            named: node
                .inner
                .iter()
                .filter_map(|(name, value)| name.as_ref().map(|name| (name, value)))
                .map(|(name, value)| -> LowerResult<(_, _)> {
                    let name = ir::Identifier::desugar(name, context)?;
                    let value = ty::Type::desugar(value, context)?;
                    Ok((name, value))
                })
                .collect::<Result<Vec<(_, _)>, _>>()?,
            positional: node
                .inner
                .iter()
                .filter_map(|(name, value)| name.is_none().then_some(value))
                .map(|value| ty::Type::desugar(value, context))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
