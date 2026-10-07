// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_transform]` attribute macro.
//!
//! The macro is used in pairs, a struct and a bodyless function.
//! The struct must be convertible into an `AffineTransform`:
//!
//! ```rs
//! /// Translate a model.
//! #[builtin_transform]
//! pub struct Translate {
//!     pub x: Length,
//!     pub y: Length,
//!     pub z: Length,
//! }
//!
//! impl From<Translate> for AffineTransform {
//!     fn from(Translate { x, y, z }: Translate) -> Self {
//!         AffineTransform::Translation { x, y, z }
//!     }
//! }
//!
//! /// Translate a model by `x`, `y` and `z`.
//! #[builtin_transform]
//! pub fn translate(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;
//! ```
//!
//! The struct defines the signature and implements `Builtin` and `BuiltinOperationCall`.
//! It does not implement `BuiltinConstruct`, because the resulting model is an affine transform.
//! The function is named after the struct in snake case and its body calls
//! `BuiltinOperationCall::call`.

use crate::prelude::*;

// Parses the syntax inside #[builtin_transform(mod = #mod_name)].
//
// The `mod = #mod_name` part is injected by `#[builtin_mod]`.
struct BuiltinTransformSig {
    /// E.g. `ops`
    mod_name: Option<Ident>,
}

impl Parse for BuiltinTransformSig {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            mod_name: match input.is_empty() {
                true => None,
                false => Some(helpers::parse_mod_name(input)?),
            },
        })
    }
}

pub(crate) fn builtin_transform_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ItemStruct>(item.clone()) {
        Ok(input_struct) => builtin_transform_struct(attr, input_struct),
        Err(_) => helpers::expand_operation_call_fn("builtin_transform", attr, item),
    }
}

fn builtin_transform_struct(attr: TokenStream, input_struct: ItemStruct) -> TokenStream {
    let BuiltinTransformSig { mod_name } = parse_macro_input!(attr as BuiltinTransformSig);

    let struct_name = &input_struct.ident;

    let Some(mod_name) = mod_name else {
        return helpers::missing_mod_error("builtin_transform", struct_name);
    };

    let (field_names, params) = match helpers::struct_params(&input_struct.fields) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    let fn_name = helpers::ident_snake(struct_name);
    let static_name = helpers::ident_upper(&fn_name);
    let doc_comment = helpers::attr_fetch_doc(&input_struct.attrs);
    let id_doc = helpers::static_id_doc(&mod_name, &fn_name);

    quote! {
        #id_doc
        #input_struct

        pub static #static_name: &crate::BuiltinItem = <#struct_name as crate::Builtin>::ITEM;

        impl crate::Builtin for #struct_name {
            const ITEM: &'static crate::BuiltinItem = &crate::BuiltinItem::Operation(crate::BuiltinOperation::new(
                crate::builtin_info!(#doc_comment #mod_name::#fn_name),
                || microcad_lang_types::function_type!((
                    self: microcad_lang_types::Type::model()
                    #(, #params)*
                ) -> microcad_lang_types::Type::model()),
                #fn_name,
            ));
        }

        impl crate::BuiltinOperationCall for #struct_name {
            fn call(
                args: microcad_lang_types::Arguments,
                _ctx: &mut crate::BuiltinEvalContext,
            ) -> Result<microcad_lang_types::ModelTree, crate::BuiltinError> {
                let self_: std::rc::Rc<microcad_lang_types::ModelTree> = args.try_get("self")?;
                let transform = microcad_lang_types::math::AffineTransform::from(Self {
                    #(#field_names: args.try_get(stringify!(#field_names))?,)*
                });
                let mut tree = microcad_lang_types::ModelTree::new(
                    microcad_lang_types::Model::from(transform).with_op_properties(args),
                );
                tree.append(std::rc::Rc::unwrap_or_clone(self_));
                Ok(tree)
            }
        }
    }
    .into()
}
