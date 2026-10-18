// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model attribute evaluation

use std::rc::Rc;

use mucad_lang_base::{Identifier, PushIssue, Refer, SrcRef, SrcReferrer};
use mucad_lang_resolve::library::symbol::{self, ExprSpec};
use mucad_lang_types::{
    Color, Value, ValueError,
    model::{Attribute, Attributes},
};

use crate::{Eval, EvalContext, EvalErrorKind, EvalResult};

/// Attach model attributes, e.g. `#[color = "red"]`, to the root of a model.
///
/// A value without a model, e.g. of an `if` without `else`, stays unchanged.
/// Other values cannot have attributes.
pub(crate) fn attach_attributes(
    attr: &symbol::ModelAttributes,
    value: Value,
    context: &mut EvalContext,
) -> EvalResult<Value> {
    let symbol::ModelAttributes {
        color,
        resolution,
        name,
        // Layers are not supported by models yet.
        layer: _,
    } = attr;
    if *attr == symbol::ModelAttributes::default() || value.is_none() {
        return Ok(value);
    }

    let Value::Model(mut tree) = value else {
        let src_ref = [color, resolution, name]
            .into_iter()
            .flatten()
            .next()
            .map_or(SrcRef::none(), |expr| expr.src_ref());
        context.push_err(EvalErrorKind::CannotAttachAttributes {
            src_ref,
            value: value.to_string(),
        });
        return Ok(value);
    };

    let attributes: Attributes = attr.eval(context)?;
    let model = Rc::make_mut(&mut tree).root_model_mut();
    attributes
        .0
        .into_iter()
        .for_each(|attribute| model.attr.set(attribute));
    // The name is no attribute of the model.
    if let Some(expr) = name
        && let Some(name) = attribute_value::<String>("name", Some(expr), context)
    {
        model.name = Some(Identifier(Refer::new(name.into(), expr.src_ref())));
    }
    Ok(Value::Model(tree))
}

/// Evaluate the attributes of a model, e.g. `#[color = "red"]`.
///
/// Invalid values are reported and skipped.
impl Eval<Attributes> for symbol::ModelAttributes {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<Attributes> {
        let mut attributes = Attributes::default();
        if let Some(color) = attribute_value::<Color>("color", self.color.as_ref(), context) {
            attributes.set(Attribute::Color(color));
        }
        if let Some(resolution) = attribute_value("resolution", self.resolution.as_ref(), context) {
            attributes.set(Attribute::Resolution(resolution));
        }
        Ok(attributes)
    }
}

/// The value of an attribute converted to `T`, or `None` if it is missing or invalid.
fn attribute_value<T: TryFrom<Value, Error = ValueError>>(
    name: &'static str,
    expr: Option<&symbol::ConstantExpression>,
    context: &mut EvalContext,
) -> Option<T> {
    let expr = expr?;
    // Attribute values are evaluated while resolving.
    let Some(value) = expr.value() else {
        context.push_err(EvalErrorKind::ConstantExpressionExpected {
            src_ref: expr.src_ref(),
        });
        return None;
    };
    T::try_from(value.clone())
        .map_err(|err| {
            context.push_err(EvalErrorKind::InvalidAttributeValue {
                name,
                src_ref: expr.src_ref(),
                err,
            })
        })
        .ok()
}
