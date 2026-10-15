// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Desugar, LowerContext, LowerError, LowerResult, ir};

use microcad_lang_base::{Refer, SpanToSrcRef};
use microcad_lang_parse::ast;
use microcad_lang_types::{Integer, Quantity, Scalar};

impl Desugar<ast::Literal> for ir::ConstantValue {
    fn desugar(node: &ast::Literal, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(match &node.literal {
            ast::LiteralKind::Bool(lit) => {
                ir::ConstantValue::new(lit.value).with_src_ref(context.span_to_src_ref(&lit.span))
            }
            ast::LiteralKind::Integer(lit) => {
                ir::ConstantValue::new(Integer::from_str(lit.value.as_str()).expect(
                    "No error expected, this string already has been checked in the parse stage.",
                ))
                .with_src_ref(context.span_to_src_ref(&lit.span))
            }
            ast::LiteralKind::Float(lit) => {
                ir::ConstantValue::new(Scalar::from_str(lit.value.as_str()).expect(
                    "No error expected, this string already has been checked in the parse stage.",
                ))
                .with_src_ref(context.span_to_src_ref(&lit.span))
            }
            ast::LiteralKind::Quantity(lit) => {
                let unit = ir::Unit::desugar(&lit.unit, context)?;
                ir::ConstantValue::new(
                    Quantity::new(
                        // Normalized before rounding to a `Scalar`, e.g. `0.04m` is exactly `40mm`.
                        unit.normalize_f64(lit.value.parse::<f64>().expect("No error expected, this string already has been checked in the parse stage.")),
                        unit.quantity_type()).with_unit(unit)).with_src_ref(
                    context.span_to_src_ref(&lit.span),
                )
            }
            ast::LiteralKind::String(lit) => ir::ConstantValue::new(lit.content.clone())
                .with_src_ref(context.span_to_src_ref(&lit.span)),
            ast::LiteralKind::Error(e) => {
                return Err(LowerError::InvalidLiteral {
                    error: e.kind.clone(),
                    src_ref: context.span_to_src_ref(&e.span),
                });
            }
        })
    }
}

impl Desugar<ast::Unit> for ir::Unit {
    fn desugar(node: &ast::Unit, context: &mut LowerContext) -> LowerResult<Self> {
        use std::str::FromStr;
        ir::Unit::from_str(node.name.as_str()).map_err(|_| {
            LowerError::UnknownUnit(Refer::new(
                node.name.to_string(),
                context.span_to_src_ref(&node.span),
            ))
        })
    }
}

impl Desugar<Option<ast::Unit>> for ir::Unit {
    fn desugar(node: &Option<ast::Unit>, context: &mut LowerContext) -> LowerResult<Self> {
        Ok(node
            .as_ref()
            .map(|unit| Self::desugar(unit, context))
            .transpose()?
            .unwrap_or_default())
    }
}
