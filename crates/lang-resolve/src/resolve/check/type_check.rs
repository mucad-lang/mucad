// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Type check of parameters and the inference of their types.

use mucad_builtin::BuiltinLibrary;
use mucad_lang_base::{IssueList, PushIssue, SrcReferrer};
use mucad_lang_lower::ir;
use mucad_lang_types::Type;

use super::Check;
use crate::{DeriveType, Library, LibraryCache, ResolveErrorKind, ResolveIssue, TypeContext};

/// Give the parameters without type the type of their default value, if it is known,
/// e.g. `Length` for `radius = 1mm`. An explicit `Any` stays.
///
/// Call it after the paths have been resolved, which finds the symbols of the default values.
pub(crate) fn infer_parameter_types(
    lib: &mut Library,
    builtins: &BuiltinLibrary,
    lib_cache: &LibraryCache,
) {
    // Derive the types with the unchanged library, then set them.
    let inferred: Vec<_> = {
        let ctx = TypeContext::new(lib, builtins).with_lib_cache(lib_cache);
        lib.root
            .descendants(&lib.arena)
            .map(|node| {
                let types: Vec<Vec<Option<ir::Type>>> = lib.arena[node]
                    .get()
                    .def
                    .parameter_lists()
                    .into_iter()
                    .map(|list| {
                        list.parameters
                            .iter()
                            .map(|param| match (&param.ty, &param.default_value) {
                                (None, Some(default)) => Some(default.derive_type(&ctx))
                                    .filter(|ty| *ty != Type::Any)
                                    .map(|ty| ir::Type {
                                        ty,
                                        src_ref: default.src_ref(),
                                    }),
                                _ => None,
                            })
                            .collect()
                    })
                    .collect();
                (node, types)
            })
            .collect()
    };

    for (node, types) in inferred {
        let lists = lib.arena[node].get_mut().def.parameter_lists_mut();
        for (list, types) in lists.into_iter().zip(types) {
            for (param, ty) in list.parameters.iter_mut().zip(types) {
                if ty.is_some() {
                    param.ty = ty;
                }
            }
        }
    }
}

/// Checks the types of parameters: a parameter needs a type or a default value,
/// a default value must match the type.
#[derive(Default)]
pub(crate) struct TypeCheck(IssueList<ResolveIssue>);

impl Check for TypeCheck {
    /// Check the parameters of all workbenches and functions of `library`.
    ///
    /// Default values of symbols of other libraries have no known type here, so they match.
    fn check(library: &Library) -> IssueList<ResolveIssue> {
        let builtins = BuiltinLibrary::new();
        let ctx = TypeContext::new(library, &builtins);

        let mut type_check = Self::default();
        for node in library.root.descendants(&library.arena) {
            for list in library.arena[node].get().def.parameter_lists() {
                list.parameters
                    .iter()
                    .for_each(|param| type_check.parameter(param, &ctx));
            }
        }
        type_check.0
    }
}

impl TypeCheck {
    fn parameter(&mut self, param: &ir::Parameter, ctx: &TypeContext) {
        match (&param.ty, &param.default_value) {
            // `Any` must be written, e.g. `x: Any`.
            (None, None) => self.0.push_err(ResolveErrorKind::ParameterWithoutType {
                name: param.id.to_string(),
                src_ref: param.id.src_ref(),
            }),
            (Some(ty), Some(default)) => {
                let default_ty = default.derive_type(ctx);
                if !default_ty.matches(&ty.ty) {
                    self.0.push_err(ResolveErrorKind::TypeMismatch {
                        specified: ty.ty.clone(),
                        specified_src_ref: ty.src_ref,
                        actual: default_ty,
                        actual_src_ref: default.src_ref(),
                    })
                }
            }
            // Without known type of the default value, its type is checked when calling.
            (None, Some(_)) | (Some(_), None) => {}
        }
    }
}
