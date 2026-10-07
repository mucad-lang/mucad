// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_op(...)]` attribute macro.
//!
//! The macro is used in pairs, a struct and a bodyless function:
//!
//! ```rs
//! /// Linear extrusion of a 2D model.
//! #[builtin_op(Geometry2D -> Geometry3D)]
//! pub struct Extrude {
//!     /// Height in mm.
//!     pub height: Length,
//! }
//!
//! /// Linear extrusion of a 2D model.
//! #[builtin_op]
//! pub fn extrude(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;
//! ```
//!
//! The struct defines the signature and implements `Builtin`, `BuiltinConstruct` and
//! `BuiltinOperationCall`. The function is named after the struct in snake case and its body
//! calls `BuiltinOperationCall::call`.

use crate::prelude::*;

// Parses the syntax inside #[builtin_op(#input_type -> #output_type, mod = #mod_name)].
//
// The `mod = #mod_name` part is injected by `#[builtin_mod]`.
struct BuiltinOpSig {
    /// Model type of `self`, e.g. `Geometry2D`.
    input_type: Ident,
    /// Model type of the result, e.g. `Geometry3D`.
    output_type: Ident,
    /// E.g. `ops`
    mod_name: Option<Ident>,
}

impl Parse for BuiltinOpSig {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_type: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let output_type: Ident = input.parse()?;

        let mod_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(helpers::parse_mod_name(input)?)
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

pub(crate) fn builtin_op_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ItemStruct>(item.clone()) {
        Ok(input_struct) => builtin_op_struct(attr, input_struct),
        Err(_) => helpers::expand_operation_call_fn("builtin_op", attr, item),
    }
}

fn builtin_op_struct(attr: TokenStream, input_struct: ItemStruct) -> TokenStream {
    let BuiltinOpSig {
        input_type,
        output_type,
        mod_name,
    } = parse_macro_input!(attr as BuiltinOpSig);

    let struct_name = &input_struct.ident;

    let Some(mod_name) = mod_name else {
        return helpers::missing_mod_error("builtin_op", struct_name);
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
                    self: microcad_lang_types::Type::Model(microcad_lang_types::ModelType::#input_type)
                    #(, #params)*
                ) -> microcad_lang_types::Type::Model(microcad_lang_types::ModelType::#output_type)),
                #fn_name,
            ));
        }

        impl crate::BuiltinConstruct for #struct_name {
            fn element() -> microcad_lang_types::model::Element {
                microcad_lang_types::model::BuiltinWorkpiece::Operation(
                    <Self as crate::Builtin>::ITEM.id()
                ).into()
            }

            fn from_model(model: &microcad_lang_types::Model) -> Result<Self, crate::BuiltinError> {
                <Self as crate::BuiltinConstruct>::check_element(model)?;
                Ok(Self {
                    #(#field_names: model.get_property_as(stringify!(#field_names))?,)*
                })
            }
        }

        impl crate::BuiltinOperationCall for #struct_name {
            fn call(
                args: microcad_lang_types::Arguments,
                _ctx: &mut crate::BuiltinEvalContext,
            ) -> Result<microcad_lang_types::ModelTree, crate::BuiltinError> {
                let self_: std::rc::Rc<microcad_lang_types::ModelTree> = args.try_get("self")?;
                let mut tree = microcad_lang_types::ModelTree::new(
                    microcad_lang_types::Model::new(
                        <Self as crate::BuiltinConstruct>::element()
                    ).with_op_properties(args),
                );
                tree.append(std::rc::Rc::unwrap_or_clone(self_));
                Ok(tree)
            }
        }
    }
    .into()
}
