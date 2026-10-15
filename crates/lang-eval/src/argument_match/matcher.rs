// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Match the arguments of a call to parameters.

use microcad_lang_base::{Identifier, IdentifierList};
use microcad_lang_types::{
    ArgumentValue, ArgumentValueList, Arguments, CallParameters, Tuple, Type, Value,
};

use crate::{ArgumentMatchError, MatchPriority};

/// A parameter with the value of its argument.
struct MatchedParameter {
    id: Identifier,
    ty: Type,
    /// The value of the argument or the default value.
    value: Option<Value>,
    /// The argument has been given, its value is no default value.
    given: bool,
}

/// Matches the arguments of a call to parameters in three steps:
///
/// 1. Named arguments by the parameter name, or by an abbreviation (e.g. `r` for `radius`).
/// 2. Positional arguments by inline identifier or by type,
///    an argument without matching parameter is an error.
/// 3. Default values for parameters without arguments.
pub(super) struct Matcher {
    params: Vec<MatchedParameter>,
    priority: MatchPriority,
}

impl Matcher {
    /// A matcher for `parameters`, which get their `default_values` if no argument is given.
    pub(super) fn new(parameters: &CallParameters, default_values: &Tuple) -> Self {
        Self {
            params: parameters
                .iter()
                .map(|(id, ty)| MatchedParameter {
                    id: id.clone(),
                    ty: ty.clone(),
                    value: default_values.get_field(id).cloned(),
                    given: false,
                })
                .collect(),
            priority: MatchPriority::default(),
        }
    }

    /// Match `arguments` and return the arguments by parameter name.
    pub(super) fn match_arguments(
        mut self,
        arguments: &ArgumentValueList,
    ) -> Result<Arguments, ArgumentMatchError> {
        self.named(arguments)?;
        self.positional(arguments)?;
        self.finish()
    }

    /// The priority of the match of `arguments`, see [`MatchPriority`].
    pub(super) fn match_priority(
        mut self,
        arguments: &ArgumentValueList,
    ) -> Result<MatchPriority, ArgumentMatchError> {
        self.named(arguments)?;
        self.positional(arguments)?;
        let priority = self.priority;
        self.finish().map(|_| priority)
    }

    /// Match named arguments by the parameter name or an abbreviation.
    fn named(&mut self, arguments: &ArgumentValueList) -> Result<(), ArgumentMatchError> {
        let mut unexpected = Vec::new();
        for (id, value) in arguments
            .iter()
            .filter_map(|ArgumentValue { id, value, .. }| Some((id.as_ref()?, value)))
        {
            match self.find(id)? {
                Some(param) if param.given => {
                    return Err(ArgumentMatchError::DuplicateArgument(param.id.clone()));
                }
                Some(param) => {
                    param.value = Some(value.clone());
                    param.given = true;
                }
                None => unexpected.push(id.clone()),
            }
        }

        match unexpected.is_empty() {
            true => Ok(()),
            false => Err(ArgumentMatchError::UnexpectedArguments(
                IdentifierList::from_iter(unexpected),
            )),
        }
    }

    /// The parameter for the named argument `id`.
    ///
    /// This is the parameter with the same name or, if `id` is a single letter,
    /// the only parameter starting with this letter, e.g. `r` for `radius`.
    fn find(
        &mut self,
        id: &Identifier,
    ) -> Result<Option<&mut MatchedParameter>, ArgumentMatchError> {
        if let Some(position) = self.params.iter().position(|param| &param.id == id) {
            return Ok(Some(&mut self.params[position]));
        }

        let mut letters = id.as_str().chars();
        let (Some(letter), None) = (letters.next(), letters.next()) else {
            return Ok(None);
        };
        let candidates: Vec<_> = self
            .params
            .iter()
            .enumerate()
            .filter(|(_, param)| param.id.as_str().starts_with(letter))
            .map(|(position, _)| position)
            .collect();

        match candidates.as_slice() {
            [] => Ok(None),
            [position] => Ok(Some(&mut self.params[*position])),
            _ => Err(ArgumentMatchError::AmbiguousArgument {
                argument: id.clone(),
                parameters: candidates
                    .iter()
                    .map(|position| self.params[*position].id.clone())
                    .collect(),
            }),
        }
    }

    /// Match positional arguments by type to parameters without an argument.
    ///
    /// A single identifier as argument (e.g. `x` in `f(x)`) matches a parameter with the same
    /// name, otherwise it is matched by type like any positional argument, but never as short name.
    /// Required parameters are filled before parameters with default values, e.g. in
    /// `assert_eq(a: Any, b: Any, message: String = "")` two strings are `a` and `b`.
    /// Within these, a parameter of exactly the argument type is preferred, e.g. to `Any`,
    /// and a list matches a parameter of its element type last (parameter multiplicity).
    /// Positional arguments, which match no parameter, are an error.
    fn positional(&mut self, arguments: &ArgumentValueList) -> Result<(), ArgumentMatchError> {
        use microcad_lang_types::Ty;

        // Inline identifiers first claim the parameter with the same name, e.g. `x` in `f(x)`.
        let mut by_type = Vec::new();
        for arg in arguments.iter().filter(|arg| arg.id.is_none()) {
            let position = arg.inline_id.as_ref().and_then(|id| {
                self.params
                    .iter()
                    .position(|param| !param.given && &param.id == id)
            });
            match position {
                Some(position) => {
                    self.give(position, &arg.value);
                    self.priority.inline_names += 1;
                }
                None => by_type.push(&arg.value),
            }
        }

        let mut unexpected = Vec::new();
        for value in by_type {
            let ty = value.ty();
            // Required parameters come first, then those with default values.
            // Within these, the exact type is preferred, e.g. to `Any`, multiplicity comes last.
            let open = |param: &MatchedParameter, required: bool| {
                !param.given && param.value.is_none() == required
            };
            let position = [true, false].into_iter().find_map(|required| {
                self.params
                    .iter()
                    .position(|param| open(param, required) && param.ty == ty)
                    .or_else(|| {
                        self.params
                            .iter()
                            .position(|param| open(param, required) && ty.matches(&param.ty))
                    })
                    .or_else(|| {
                        self.params.iter().position(|param| {
                            open(param, required) && ty.matches_multiplicity(&param.ty)
                        })
                    })
                    .map(|position| (position, required))
            });
            match position {
                Some((position, required)) => {
                    self.give(position, value);
                    self.priority.required += usize::from(required);
                }
                // The type shows why the argument does not match, e.g. `1 (Scalar)`.
                None => unexpected.push(format!("{value} ({ty})")),
            }
        }

        match unexpected.is_empty() {
            true => Ok(()),
            false => Err(ArgumentMatchError::UnexpectedPositionalArguments(
                unexpected.join(", "),
            )),
        }
    }

    /// Give the argument `value` to the parameter at `position`.
    fn give(&mut self, position: usize, value: &Value) {
        let param = &mut self.params[position];
        param.value = Some(value.clone());
        param.given = true;
    }

    /// The arguments by parameter name, or an error for parameters without value.
    fn finish(self) -> Result<Arguments, ArgumentMatchError> {
        let missing: Vec<_> = self
            .params
            .iter()
            .filter(|param| param.value.is_none())
            .map(|param| param.id.clone())
            .collect();
        if !missing.is_empty() {
            return Err(ArgumentMatchError::MissingArguments(
                IdentifierList::from_iter(missing),
            ));
        }

        Ok(Arguments::from_iter(
            self.params
                .into_iter()
                .map(|param| (param.id, param.value.unwrap_or_default())),
        ))
    }
}
