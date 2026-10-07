// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_primitive(...)]` attribute macro.

use crate::prelude::*;

// Parses the syntax inside #[builtin_primitive(#model_type, mod = #mod_name)]
//
// The `mod = #mod_name` part is injected by `#[builtin_mod]`.
struct BuiltinPrimitiveSig {
    /// E.g. `Geometry2D`
    model_type: Ident,
    /// E.g. `geo2d`
    mod_name: Option<Ident>,
}

impl Parse for BuiltinPrimitiveSig {
    fn parse(input: ParseStream) -> Result<Self> {
        let model_type: Ident = input.parse()?;

        let mod_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse::<Token![mod]>()?;
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            model_type,
            mod_name,
        })
    }
}

pub(crate) fn builtin_primitive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the macro attribute syntax
    let BuiltinPrimitiveSig {
        model_type,
        mod_name,
    } = parse_macro_input!(attr as BuiltinPrimitiveSig);

    // Parse the annotated struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &input_struct.ident;

    let Some(mod_name) = mod_name else {
        return syn::Error::new_spanned(
            struct_name,
            "#[builtin_primitive] must be used inside a #[builtin_mod] module",
        )
        .to_compile_error()
        .into();
    };

    let (field_names, params) = match helpers::struct_params(&input_struct.fields) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    let static_name = helpers::ident_upper(struct_name);
    let doc_comment = helpers::attr_fetch_doc(&input_struct.attrs);

    quote! {
        #input_struct

        pub static #static_name: &BuiltinItem = #struct_name::ITEM;

        impl BuiltinConstruct for #struct_name {
            const ITEM: &'static BuiltinItem = &builtin_item!(
                Primitive
                #doc_comment
                #mod_name::#struct_name(microcad_lang_types::function_type!(
                    (#(#params),*) -> microcad_lang_types::Type::Model(microcad_lang_types::ModelType::#model_type)
                ))
            );

            fn from_model(model: &microcad_lang_types::Model) -> Result<Self, BuiltinError> {
                Self::check_element(model)?;
                Ok(Self {
                    #(#field_names: model.get_property_as(stringify!(#field_names))?,)*
                })
            }
        }

        impl BuiltinPrimitiveCall for #struct_name {}
    }
    .into()
}
