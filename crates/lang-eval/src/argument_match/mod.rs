// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Argument match trait

use microcad_lang_base::{Identifier, IdentifierList};
use microcad_lang_resolve::library::symbol::{
    Function, ParameterList, function::FunctionSignature, workbench::Init,
};
use microcad_lang_types::{ArgumentValueList, Arguments, CallSignature, Tuple, Type, Value};
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ArgumentMatchError {
    /// Multiplicity (variadic expansion) is not supported for variadic call signatures.
    #[error("Multiplicity `({arguments})` is not allowed for variadic call signatures")]
    #[diagnostic(help(
        "Pass arguments explicitly instead of using variadic expansion on multi-matches."
    ))]
    MultiMatchNotAllowedForVariadic {
        /// Name of the symbol or call signature
        arguments: String,
    },

    /// Missing arguments
    #[error("Missing arguments: {0}")]
    MissingArguments(IdentifierList),

    /// Unexpected arguments
    #[error("Unexpected arguments: {0}")]
    UnexpectedArguments(IdentifierList),
}

pub trait ArgumentMatch {
    fn call_signature(&self) -> CallSignature;

    fn default_values(&self) -> Tuple {
        Tuple::default()
    }

    fn is_matching(&self, arguments: &ArgumentValueList) -> bool {
        self.argument_match(arguments).is_ok() // TODO Implement a custom and more performant algorithm here.
    }

    fn argument_match(
        &self,
        arguments: &ArgumentValueList,
    ) -> Result<Arguments, ArgumentMatchError> {
        use microcad_lang_types::Ty;

        let call_signature = self.call_signature();

        // 1. A variadic function simply returns the arguments as tuple.
        if call_signature.is_variadic() {
            return Ok(Arguments::from(Tuple {
                named: arguments
                    .iter()
                    .filter_map(|arg| arg.id.as_ref().map(|id| (id.clone(), arg.value.clone())))
                    .collect(),
                positional: arguments
                    .iter()
                    .filter_map(|arg| match &arg.id {
                        Some(_) => None,
                        None => Some(arg.value.clone()),
                    })
                    .collect(),
            }));
        }

        let params = &call_signature.parameters.unwrap();
        let default_values = self.default_values();

        #[derive(Debug)]
        struct MatchArg {
            id: Identifier,
            ty: Type,
            value: Option<Value>,
        }

        // 2. Construct the named arguments first, do the type check later
        let mut matched_args: Vec<MatchArg> = params
            .iter()
            .map(|(id, ty)| MatchArg {
                id: id.clone(),
                ty: ty.clone(),
                value: default_values.get_field(id).cloned(),
            })
            .collect();

        let mut unexpected_arguments = Vec::new();

        // 3. Try to match named arguments
        arguments.args.iter().for_each(|arg| {
            if let Some(id) = &arg.id {
                if let Some(match_arg) = matched_args
                    .iter_mut()
                    .find(|match_arg| &match_arg.id == id)
                {
                    match_arg.value = Some(arg.value.clone());
                } else {
                    unexpected_arguments.push(id.clone());
                }
            }
        });

        if !unexpected_arguments.is_empty() {
            return Err(ArgumentMatchError::UnexpectedArguments(
                IdentifierList::from_iter(unexpected_arguments),
            ));
        }

        // 4. Try to find positional arguments by type
        arguments.args.iter().for_each(|arg| {
            if arg.id.is_none()
                && let Some(ref mut match_arg) = matched_args
                    .iter_mut()
                    .filter(|match_arg| match_arg.value.is_none())
                    .find(|match_arg| match_arg.ty == arg.value.ty())
            {
                match_arg.value = Some(arg.value.clone());
            }
        });

        let missing_arguments: Vec<_> = matched_args
            .iter()
            .filter_map(|arg| {
                if arg.value.is_none() {
                    Some(arg.id.clone())
                } else {
                    None
                }
            })
            .collect();

        if !missing_arguments.is_empty() {
            return Err(ArgumentMatchError::MissingArguments(
                IdentifierList::from_iter(missing_arguments),
            ));
        }

        Ok(Arguments::from_iter(matched_args.into_iter().map(
            |match_arg| (match_arg.id, match_arg.value.unwrap_or_default()),
        )))
    }

    fn argument_multi_match(
        &self,
        arguments: &ArgumentValueList,
    ) -> Result<Vec<Arguments>, ArgumentMatchError> {
        // Step 1: Bind base arguments and resolve default values
        let base_args = self.argument_match(arguments)?;

        let call_signature = self.call_signature();
        if call_signature.is_variadic() {
            return Err(ArgumentMatchError::MultiMatchNotAllowedForVariadic {
                arguments: arguments.to_string(),
            });
        }

        // Step 2: Extract parameter metadata
        let params_map = call_signature.parameters.unwrap();

        // Step 3: Identify which arguments need vectorization
        let mut resolved_lists: Vec<(Identifier, Vec<Value>)> =
            Vec::with_capacity(base_args.0.named.len());

        for (id, val) in base_args.0.named {
            let expected_ty = params_map
                .iter()
                .find(|(p_id, _)| p_id == &id)
                .map(|(_, ty)| ty);

            if let (Some(expected), Value::List(elements)) = (expected_ty, &val) {
                if expected.is_list_of(expected) || !expected.is_list_of(&Type::Any) {
                    resolved_lists.push((id, elements.to_vec()));
                } else {
                    resolved_lists.push((id, vec![val.clone()]));
                }
            } else {
                resolved_lists.push((id, vec![val.clone()]));
            }
        }

        // Step 4: Compute Cartesian product across all argument value lists
        let mut combinations: Vec<Vec<(Identifier, Value)>> = vec![vec![]];

        for (id, values) in resolved_lists {
            let mut next_combinations = Vec::new();
            for current_combination in combinations {
                for val in &values {
                    let mut extended = current_combination.clone();
                    extended.push((id.clone(), val.clone()));
                    next_combinations.push(extended);
                }
            }
            combinations = next_combinations;
        }

        // Step 5: Convert combinations into Arguments
        let result = combinations
            .into_iter()
            .map(|named| {
                Tuple {
                    positional: vec![],
                    named,
                }
                .into()
            })
            .collect();

        Ok(result)
    }
}

impl ArgumentMatch for ParameterList {
    /// Return default values for this parameters, assuming all constant expression have been folded into values.
    fn default_values(&self) -> Tuple {
        use microcad_lang_resolve::library::symbol::ExprSpec;
        Tuple::from_iter(self.parameters.iter().filter_map(|param| {
            param
                .default_value
                .as_ref()
                .and_then(|expr| expr.value())
                .map(|value| (param.id.clone(), value.clone()))
        }))
    }

    /// Parameters call signature
    fn call_signature(&self) -> CallSignature {
        use microcad_lang_types::Ty;
        CallSignature::new(
            self.iter()
                .map(|param| (param.id.clone(), param.ty()))
                .collect::<Vec<_>>(),
        )
    }
}

impl ArgumentMatch for Init {
    fn call_signature(&self) -> CallSignature {
        self.parameters.call_signature()
    }

    fn default_values(&self) -> Tuple {
        self.parameters.default_values()
    }
}

impl ArgumentMatch for FunctionSignature {
    fn call_signature(&self) -> CallSignature {
        self.parameters.call_signature()
    }

    fn default_values(&self) -> Tuple {
        self.parameters.default_values()
    }
}

impl ArgumentMatch for Function {
    fn call_signature(&self) -> CallSignature {
        self.signature.call_signature()
    }

    fn default_values(&self) -> Tuple {
        self.signature.default_values()
    }
}

impl ArgumentMatch for microcad_builtin::BuiltinFunction {
    fn call_signature(&self) -> CallSignature {
        self.ty().call_ty.clone()
    }
}

impl ArgumentMatch for microcad_builtin::BuiltinPrimitive {
    fn call_signature(&self) -> CallSignature {
        self.ty().call_ty.clone()
    }
}

impl ArgumentMatch for microcad_builtin::BuiltinOperation {
    fn call_signature(&self) -> CallSignature {
        (self.ty)().call_ty.clone()
    }
}
