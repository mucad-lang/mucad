// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Workbench evaluation

use crate::{
    Callable, Eval, EvalContext, EvalError, EvalErrorKind, EvalResult,
    context::{
        BuiltinItemFrame, ContextScope, WorkbenchGroupFrame, WorkbenchInitFrame, WorkpieceFrame,
    },
};

use microcad_builtin::BuiltinItem;
use microcad_lang_base::{
    DisplayWithCtx, PushIssue, SrcReferrer,
    element::{Visibility, WorkbenchKind},
};
use microcad_lang_resolve::library::symbol;

use microcad_lang_types::{
    ArgumentValue, ArgumentValueList, ModelTree, Value,
    model::{Element, ModelTreeBuilderMut, Properties, Property, PropertyType, Workpiece},
};

impl Eval for symbol::workbench::Group {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        context.scope(WorkbenchGroupFrame::new(), |context| {
            self.statements
                .iter()
                .try_for_each(|stmt| stmt.eval(context))?;
            Ok(context.model_tree_builder_mut().build().into())
        })
    }
}

impl Eval for symbol::workbench::WorkbenchIf {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        let cond: Value = self.cond.eval(context)?;
        let cond: bool = match cond.try_into() {
            Ok(cond) => cond,
            Err(err) => {
                return context.catch(EvalErrorKind::IfConditionIsNotBool {
                    condition_src_ref: self.cond.src_ref(),
                    src_ref: self.src_ref,
                    err,
                });
            }
        };

        if cond {
            self.body.eval(context)
        } else if let Some(next_if) = &self.next_if {
            // Handle `else if ...` chain
            next_if.eval(context)
        } else if let Some(body) = &self.body_else {
            body.eval(context)
        } else {
            Ok(Value::None)
        }
    }
}

impl Eval<ModelTree> for symbol::workbench::Marker {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<ModelTree> {
        if !self.is_input_placeholder() {
            return Ok(ModelTree::new(Element::InputPlaceholder));
        }

        // `@input` is the input model of the operation, which will become `self` in the future.
        match context.look_up_local("self") {
            Some(input) => Ok(ModelTree::from(input.clone())),
            None => Err(EvalErrorKind::InputNotAvailable {
                src_ref: self.src_ref,
            }
            .into()),
        }
    }
}

impl Eval for BuiltinItem {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        match self {
            BuiltinItem::Constant(builtin_constant) => Ok(builtin_constant.value()),
            item => context.catch(EvalErrorKind::BuiltinConstantExpected {
                name: item.name().to_string(),
                src_ref: context.current_symbol_src_ref(),
            }),
        }
    }
}

impl Eval for symbol::Path {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        match self {
            symbol::Path::Resolved(symbol_id)
            | symbol::Path::HumanReadable { id: symbol_id, .. } => symbol_id.eval(context),
            symbol::Path::Unresolved(unresolved_path) => {
                context.catch(EvalErrorKind::UnresolvedPath {
                    path: unresolved_path.to_string(),
                    src_ref: unresolved_path.src_ref,
                })
            }
        }
    }
}

impl Eval<ArgumentValue> for symbol::workbench::Argument {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<ArgumentValue> {
        Ok(match self {
            symbol::workbench::Argument::Unnamed(expr) => {
                let value: Value = expr.eval(context)?;
                ArgumentValue::new(value, None)
            }
            symbol::workbench::Argument::Named { name, expr, .. }
            | symbol::workbench::Argument::AutoNamed { name, expr } => {
                let value: Value = expr.eval(context)?;
                ArgumentValue::new(value, Some(name.clone()))
            }
        })
    }
}

impl Eval<ArgumentValueList> for symbol::workbench::ArgumentList {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<ArgumentValueList> {
        let map: Vec<ArgumentValue> = self
            .args
            .iter()
            .map(|arg| arg.eval(context))
            .collect::<EvalResult<Vec<_>>>()?;

        Ok(ArgumentValueList {
            args: map,
            src_ref: self.src_ref,
        })
    }
}

impl Eval for symbol::workbench::WorkbenchCall {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        let symbol_id = match &self.path {
            symbol::Path::Resolved(symbol_id)
            | symbol::Path::HumanReadable { id: symbol_id, .. } => Some(symbol_id),
            symbol::Path::Unresolved(_) => None,
        };

        match symbol_id {
            Some(symbol::SymbolId::Builtin(builtin_id)) => {
                let args = self.args.eval(context)?;
                let item = context.builtins().get(*builtin_id);
                // Errors of the built-in refer to this call.
                let src_ref = self.src_ref;
                match item {
                    Some(item) => context.scope(BuiltinItemFrame::new(src_ref, item), |ctx| {
                        item.call(&args, ctx)
                    }),
                    None => context.catch(EvalErrorKind::BuiltinNotFound {
                        name: builtin_id.to_string_with_ctx(context),
                        src_ref,
                    }),
                }
            }
            Some(symbol::SymbolId::Item(id)) => match context.lib_id() {
                Some(lib_id) => context.call_symbol_workbench(self, *lib_id, *id),
                None => context.catch(EvalErrorKind::NoLibraryToEvaluate),
            },
            Some(symbol::SymbolId::External(external_id)) => {
                context.call_symbol_workbench(self, external_id.lib_id, external_id.id)
            }
            _ => context.catch(EvalErrorKind::SymbolCannotBeCalled {
                path: self.path.to_string(),
                src_ref: self.src_ref,
            }),
        }
    }
}

impl Eval for symbol::WorkbenchExpression {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        match &self {
            // The error has already been reported when lowering.
            symbol::WorkbenchExpression::Invalid => Ok(Value::None),
            symbol::WorkbenchExpression::Value(constant_value) => {
                Ok(constant_value.value().clone())
            }
            symbol::WorkbenchExpression::Path(path) => path.eval(context),
            symbol::WorkbenchExpression::Group(group) => group.eval(context),
            symbol::WorkbenchExpression::If(if_) => if_.eval(context),
            symbol::WorkbenchExpression::Call(call) => call.eval(context),
            symbol::WorkbenchExpression::Marker(marker) => Ok(marker.eval(context)?.into()),
            _ => unimplemented!(),
        }
    }
}

impl Eval<()> for symbol::WorkbenchStatement {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<()> {
        match &self.name {
            // If we have an id, this becomes a property in the current model
            Some(name) => {
                let property = Property {
                    name: name.clone(),
                    value: self.expression.eval(context)?,
                    src_ref: self.src_ref,
                    ty: match &self.visibility {
                        Visibility::Public => PropertyType::Output,
                        Visibility::Private => PropertyType::Hidden,
                    },
                };
                context
                    .model_tree_builder_mut()
                    .add_model_property(property);
            }
            // If we have no id, we have a child model.
            None => {
                let value: Value = self.expression.eval(context)?;
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

/// Evaluate an [`InitStatement`] into an input property.
///
/// This does not throw an error if the property is not as workbench parameter.
impl Eval<Property> for symbol::workbench::InitStatement {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<Property> {
        let value: Value = self.expression.eval(context)?;
        Ok(Property::input(self.name.clone(), value).with_src_ref(self.src_ref))
    }
}

impl Callable for symbol::Workbench {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        fn eval_to_model(
            workbench: &symbol::Workbench,
            input_properties: Properties,
            input: Option<Value>,
            context: &mut EvalContext,
        ) -> EvalResult<ModelTree> {
            context.scope(
                WorkpieceFrame::new(Workpiece::new(workbench.signature.kind)).with_input(input),
                |context| -> EvalResult<ModelTree> {
                    context
                        .model_tree_builder_mut()
                        .add_model_properties(input_properties.into_iter().map(|(_, prop)| prop));
                    workbench
                        .statements
                        .iter()
                        .try_for_each(|stmt| stmt.eval(context))?;
                    Ok(context.model_tree_builder_mut().build())
                },
            )
        }

        // An operation takes its input model as `self`, which is not matched with the inits.
        let mut args = args.clone();
        let input = match self.signature.kind {
            WorkbenchKind::Op => match args.remove_named("self") {
                Some(input) => Some(input),
                None => {
                    return Err(EvalErrorKind::CannotCallOperationWithoutWorkpiece {
                        name: context.current_symbol_name().unwrap_or_default(),
                        src_ref: context.current_symbol_src_ref(),
                    }
                    .into());
                }
            },
            WorkbenchKind::Sketch | WorkbenchKind::Part => None,
        };
        let args = &args;

        use crate::ArgumentMatch;

        let matching_inits: Vec<_> = self
            .signature
            .inits
            .iter()
            .filter(|iter| iter.is_matching(args))
            .collect();

        match matching_inits.len() {
            0 => Err(EvalErrorKind::NoInitializationFound {
                src_ref: self.signature.parameters.src_ref,
                path: context.current_symbol_name().unwrap_or_default(),
                arguments: args.to_string(),
                inits: self
                    .signature
                    .inits
                    .iter()
                    .map(|init| init.to_string())
                    .collect(),
            }
            .into()),
            1 => {
                let init = matching_inits.first().unwrap();
                let mut models = Vec::new();

                match init.argument_multi_match(args) {
                    Ok(arguments) => {
                        for args in arguments {
                            let properties = context.scope(
                                WorkbenchInitFrame::new(args),
                                |context| -> EvalResult<Properties> {
                                    let mut properties = Properties::new();
                                    init.statements.iter().try_for_each(
                                        |stmt| -> EvalResult<()> {
                                            let property: Property = stmt.eval(context)?;
                                            properties.set_property(property);
                                            Ok(())
                                        },
                                    )?;
                                    Ok(properties)
                                },
                            )?;

                            models.push(eval_to_model(self, properties, input.clone(), context)?);
                        }
                    }
                    Err(err) => {
                        return Err(EvalError::argument_match(
                            self.signature.parameters.src_ref,
                            context.current_symbol_name().unwrap_or_default(),
                            err,
                        ));
                    }
                }

                Ok(ModelTree::to_multiplicity(models).into())
            }
            _n => Err(EvalErrorKind::AmbiguousInitialization {
                src_ref: self.signature.parameters.src_ref,
                path: context.current_symbol_name().unwrap_or_default(),
                arguments: args.to_string(),
                inits: matching_inits.iter().map(|init| init.to_string()).collect(),
            }
            .into()),
        }
    }
}
