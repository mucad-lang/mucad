// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_op(...)]` attribute macro.
//!
//! The macro is used in pairs, a struct and a function:
//!
//! ```rs
//! /// Linear extrusion of a 2D model.
//! #[builtin_op(Geometry2D -> Geometry3D)]
//! pub struct Extrude {
//!     /// Height in mm.
//!     pub height: Length,
//! }
//!
//! #[builtin_op]
//! pub fn extrude(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError> {
//!     ...
//! }
//! ```
//!
//! The struct defines the signature and implements `BuiltinConstruct` and `BuiltinOperationCall`.
//! `BuiltinOperationCall::call` calls the function, which is named after the struct in snake case.

use crate::prelude::*;

/// Parses `mod = #mod_name`, which is injected by `#[builtin_mod]`.
fn parse_mod_name(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![mod]>()?;
    input.parse::<Token![=]>()?;
    input.parse()
}

// Parses the syntax inside #[builtin_op(#input_type -> #output_type, mod = #mod_name)] for structs.
struct BuiltinOpStructSig {
    /// Model type of `self`, e.g. `Geometry2D`.
    input_type: Ident,
    /// Model type of the result, e.g. `Geometry3D`.
    output_type: Ident,
    /// E.g. `ops`
    mod_name: Option<Ident>,
}

impl Parse for BuiltinOpStructSig {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_type: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let output_type: Ident = input.parse()?;

        let mod_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(parse_mod_name(input)?)
        } else {
            None
        };

        Ok(Self {
            input_type,
            output_type,
            mod_name,
        })
    }
}

// Parses the syntax inside #[builtin_op(mod = #mod_name)] for functions.
struct BuiltinOpFnSig {
    /// E.g. `ops`
    mod_name: Option<Ident>,
}

impl Parse for BuiltinOpFnSig {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            mod_name: match input.is_empty() {
                true => None,
                false => Some(parse_mod_name(input)?),
            },
        })
    }
}

fn missing_mod_error(span: &impl quote::ToTokens) -> TokenStream {
    syn::Error::new_spanned(
        span,
        "#[builtin_op] must be used inside a #[builtin_mod] module",
    )
    .to_compile_error()
    .into()
}

pub(crate) fn builtin_op_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_macro_input!(item as syn::Item) {
        syn::Item::Struct(item_struct) => builtin_op_struct(attr, item_struct),
        syn::Item::Fn(item_fn) => builtin_op_fn(attr, item_fn),
        item => syn::Error::new_spanned(item, "#[builtin_op] can only be used on structs and functions")
            .to_compile_error()
            .into(),
    }
}

fn builtin_op_struct(attr: TokenStream, input_struct: ItemStruct) -> TokenStream {
    let BuiltinOpStructSig {
        input_type,
        output_type,
        mod_name,
    } = parse_macro_input!(attr as BuiltinOpStructSig);

    let struct_name = &input_struct.ident;

    let Some(mod_name) = mod_name else {
        return missing_mod_error(struct_name);
    };

    let (field_names, params) = match helpers::struct_params(&input_struct.fields) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    let fn_name = helpers::ident_snake(struct_name);
    let static_name = helpers::ident_upper(&fn_name);
    let doc_comment = helpers::attr_fetch_doc(&input_struct.attrs);

    quote! {
        #input_struct

        pub static #static_name: &crate::BuiltinItem = <#struct_name as crate::BuiltinConstruct>::ITEM;

        impl crate::BuiltinConstruct for #struct_name {
            const ITEM: &'static crate::BuiltinItem = &crate::BuiltinItem::Operation(crate::BuiltinOperation::new(
                crate::builtin_info!(#doc_comment #mod_name::#fn_name),
                || microcad_lang_types::function_type!((
                    self: microcad_lang_types::Type::Model(microcad_lang_types::ModelType::#input_type)
                    #(, #params)*
                ) -> microcad_lang_types::Type::Model(microcad_lang_types::ModelType::#output_type)),
                <#struct_name as crate::BuiltinOperationCall>::call,
            ));

            fn element() -> microcad_lang_types::model::Element {
                microcad_lang_types::model::BuiltinWorkpiece::Operation(Self::ITEM.id()).into()
            }

            fn from_model(model: &microcad_lang_types::Model) -> Result<Self, crate::BuiltinError> {
                Self::check_element(model)?;
                Ok(Self {
                    #(#field_names: model.get_property_as(stringify!(#field_names))?,)*
                })
            }
        }

        impl crate::BuiltinOperationCall for #struct_name {
            fn call(
                args: microcad_lang_types::Arguments,
                ctx: &mut crate::BuiltinEvalContext,
            ) -> Result<microcad_lang_types::ModelTree, crate::BuiltinError> {
                #fn_name(args, ctx)
            }
        }
    }
    .into()
}

fn builtin_op_fn(attr: TokenStream, input_fn: ItemFn) -> TokenStream {
    let BuiltinOpFnSig { mod_name } = parse_macro_input!(attr as BuiltinOpFnSig);

    let fn_name = &input_fn.sig.ident;

    let Some(mod_name) = mod_name else {
        return missing_mod_error(fn_name);
    };

    let id_doc = format!(
        "**Static ID:** `{}`",
        microcad_hash::HashId::compile_time_hash(format!("__mu::{mod_name}::{fn_name}").as_str())
    );

    quote! {
        #[doc = ""]
        #[doc = #id_doc]
        #input_fn
    }
    .into()
}
