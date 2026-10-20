// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Source evaluation

use mucad_lang_resolve::library::symbol;
use mucad_lang_types::{
    ModelTree, Value,
    model::{Attribute, ModelTreeBuilderMut, Property, PropertyType, attribute::ExportCommand},
};

use mucad_lang_base::PushIssue;

use crate::{
    Eval, EvalContext, EvalErrorKind, EvalResult,
    attribute::{attach_attributes, attach_exports},
    context::SourceFrame,
};

impl Eval<()> for symbol::SourceStatement {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<()> {
        let value = self.expression.eval(context)?;
        let value = attach_attributes(&self.attr, value, context)?;
        let value = attach_exports(&self.exports, value, context)?;
        match &self.name {
            // If we have an id, this becomes a property in the current model
            Some(name) => {
                let property = Property {
                    name: name.clone(),
                    value,
                    src_ref: self.src_ref,
                    ty: PropertyType::Hidden,
                };
                context
                    .model_tree_builder_mut()
                    .add_model_property(property);
            }
            // If we have no id, we have a child model.
            None => {
                // It might be that the value is none, because an if expression might not produce a model when its condition is not fulfilled
                if !value.is_none() {
                    context
                        .model_tree_builder_mut()
                        .add_model_child(ModelTree::from(value));
                }
            }
        }
        Ok(())
    }
}

impl Eval for symbol::Source {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        context.scope(SourceFrame::new(), |context| -> EvalResult {
            self.statements
                .iter()
                .try_for_each(|stmt| stmt.eval(context))?;
            Ok(context.model_tree_builder_mut().build().into())
        })
    }
}

impl Eval for symbol::SourceFile {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<Value> {
        match self {
            symbol::SourceFile::NotLoaded => context.catch(EvalErrorKind::SourceFileNotLoaded),
            symbol::SourceFile::Loaded { source, .. } => source.eval(context),
            // Its errors have been reported while resolving.
            symbol::SourceFile::Failed { .. } => Ok(Value::None),
        }
    }
}
