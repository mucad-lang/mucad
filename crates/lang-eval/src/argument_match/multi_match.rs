// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Match arguments with lists to multiple argument combinations.

use microcad_lang_base::Identifier;
use microcad_lang_types::{ArgumentValueList, Arguments, Tuple, Ty, Value};

use crate::{ArgumentMatch, ArgumentMatchError};

/// Match `arguments`, a list argument for a non-list parameter results in one match per element.
///
/// Returns the cartesian product of all such lists.
pub(super) fn argument_multi_match(
    matcher: &(impl ArgumentMatch + ?Sized),
    arguments: &ArgumentValueList,
) -> Result<Vec<Arguments>, ArgumentMatchError> {
    // Step 1: Bind base arguments and resolve default values
    let base_args = matcher.argument_match(arguments)?;

    let call_signature = matcher.call_signature();
    let Some(params) = call_signature.parameters else {
        return Err(ArgumentMatchError::MultiMatchNotAllowedForVariadic {
            arguments: arguments.to_string(),
        });
    };

    // Step 2: Identify which arguments need vectorization
    let resolved_lists: Vec<(Identifier, Vec<Value>)> = base_args
        .0
        .named
        .into_iter()
        .map(|(id, val)| {
            let expected_ty = params
                .iter()
                .find(|(p_id, _)| p_id == &id)
                .map(|(_, ty)| ty);
            match (expected_ty, &val) {
                (Some(expected), Value::List(elements))
                    if val.ty().matches_multiplicity(expected) =>
                {
                    let elements = elements.to_vec();
                    (id, elements)
                }
                _ => (id, vec![val]),
            }
        })
        .collect();

    // Step 3: Compute Cartesian product across all argument value lists
    let mut combinations: Vec<Vec<(Identifier, Value)>> = vec![vec![]];
    for (id, values) in resolved_lists {
        let mut next_combinations = Vec::new();
        for combination in combinations {
            for val in &values {
                let mut extended = combination.clone();
                extended.push((id.clone(), val.clone()));
                next_combinations.push(extended);
            }
        }
        combinations = next_combinations;
    }

    // Step 4: Convert combinations into Arguments
    Ok(combinations
        .into_iter()
        .map(|named| {
            Tuple {
                positional: vec![],
                named,
            }
            .into()
        })
        .collect())
}
