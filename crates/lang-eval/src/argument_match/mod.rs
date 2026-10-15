// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Match the arguments of a call to parameters.

mod matcher;
mod multi_match;

use microcad_lang_base::{Identifier, IdentifierList};
use microcad_lang_resolve::library::symbol::{
    Function, ParameterList, function::FunctionSignature, workbench::Init,
};
use microcad_lang_types::{ArgumentValue, ArgumentValueList, Arguments, CallSignature, Tuple};
use miette::Diagnostic;
use thiserror::Error;

use matcher::Matcher;

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

    /// Positional arguments, which match no parameter by type.
    #[error("Unexpected positional arguments: {0}")]
    UnexpectedPositionalArguments(String),

    /// An abbreviated argument matches several parameters.
    #[error("Argument `{argument}` is ambiguous: {parameters}")]
    #[diagnostic(help("Use the full parameter name."))]
    AmbiguousArgument {
        /// The abbreviated argument, e.g. `f`.
        argument: Identifier,
        /// The parameters starting like the argument, e.g. `foo` and `foz`.
        parameters: IdentifierList,
    },

    /// A parameter is given more than once, e.g. as `r` and `radius`.
    #[error("Argument `{0}` is given more than once")]
    DuplicateArgument(Identifier),
}

pub trait ArgumentMatch {
    fn call_signature(&self) -> CallSignature;

    fn default_values(&self) -> Tuple {
        Tuple::default()
    }

    fn is_matching(&self, arguments: &ArgumentValueList) -> bool {
        self.argument_match(arguments).is_ok() // TODO Implement a custom and more performant algorithm here.
    }

    /// Match `arguments` to the parameters of the call signature.
    ///
    /// Named arguments are matched by name or by an abbreviation (e.g. `r` for `radius`),
    /// positional arguments by type. Parameters without argument get their default value.
    fn argument_match(
        &self,
        arguments: &ArgumentValueList,
    ) -> Result<Arguments, ArgumentMatchError> {
        match self.call_signature().parameters {
            Some(parameters) => {
                Matcher::new(&parameters, &self.default_values()).match_arguments(arguments)
            }
            // A variadic function simply returns the arguments as tuple,
            // arguments with inline identifiers are named.
            None => {
                let name =
                    |arg: &ArgumentValue| arg.id.as_ref().or(arg.inline_id.as_ref()).cloned();
                Ok(Arguments::from(Tuple {
                    named: arguments
                        .iter()
                        .filter_map(|arg| name(arg).map(|id| (id, arg.value.clone())))
                        .collect(),
                    positional: arguments
                        .iter()
                        .filter(|arg| name(arg).is_none())
                        .map(|arg| arg.value.clone())
                        .collect(),
                }))
            }
        }
    }

    /// The priority of the match of `arguments`, higher is better, or `None` if they don't match.
    ///
    /// Inline identifiers (e.g. `x` in `f(x)`) matching a parameter by name have priority over
    /// matching by type, e.g. to choose between overloaded initializers.
    fn match_priority(&self, arguments: &ArgumentValueList) -> Option<usize> {
        match self.call_signature().parameters {
            Some(parameters) => Matcher::new(&parameters, &self.default_values())
                .match_priority(arguments)
                .ok(),
            None => Some(0),
        }
    }

    /// Match `arguments`, a list argument for a non-list parameter results in one match
    /// per element.
    fn argument_multi_match(
        &self,
        arguments: &ArgumentValueList,
    ) -> Result<Vec<Arguments>, ArgumentMatchError> {
        multi_match::argument_multi_match(self, arguments)
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

    fn default_values(&self) -> Tuple {
        self.default_values()
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
