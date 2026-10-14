// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_fn(...)` attribute macro.

use quote::ToTokens;

use crate::{
    helpers::{BuiltinParam, BuiltinType},
    prelude::*,
};

// Parses the syntax inside #[builtin_fn(#mod_name::#name(arg1: Type1, ...) -> #return_type)]
struct BuiltinFnSig {
    /// E.g. `core`
    mod_name: Ident,
    /// E:g. `add`
    name: Ident,
    is_variadic: bool,
    is_impure: bool,
    params: Vec<BuiltinParam>,
    return_type: Option<BuiltinType>,
}

syn::custom_keyword!(impure);

impl Parse for BuiltinFnSig {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check if the keyword `impure` exists.
        let is_impure = if input.peek(impure) {
            let _: Ident = input.parse()?;
            true
        } else {
            false
        };

        // 2. Parse `mod_name::name`
        let mod_name: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let name: Ident = input.parse()?;

        // 3. Parse parenthesized parameter list `(arg1: Type1, ...)`
        let content;
        syn::parenthesized!(content in input);

        let mut params = Vec::new();
        let mut is_variadic = false;

        while !content.is_empty() {
            if content.peek(syn::token::Star) {
                is_variadic = true;
                content.parse::<Token![*]>()?;
                break;
            }
            params.push(content.parse::<BuiltinParam>()?);

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        // 3. Optionally Parse `-> return_type`
        let return_type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            mod_name,
            name,
            params,
            return_type,
            is_variadic,
            is_impure,
        })
    }
}

pub(crate) fn builtin_fn_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the macro attribute syntax
    let BuiltinFnSig {
        name,
        mod_name,
        params,
        return_type,
        is_variadic,
        is_impure,
    } = parse_macro_input!(attr as BuiltinFnSig);

    // Parse the annotated function
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;

    if name != *fn_name {
        return syn::Error::new_spanned(
            &name,
            format!(
                "builtin function name `{}` does not match Rust function name `{}`",
                name, fn_name
            ),
        )
        .to_compile_error()
        .into();
    }

    let static_name = helpers::ident_upper(fn_name);

    // Generate list of parameters from input tokens.
    let defaults: Vec<_> = params
        .iter()
        .filter_map(BuiltinParam::default_tokens)
        .collect();
    let defaults = match defaults.is_empty() {
        true => quote! {},
        false => quote! { defaults(#(#defaults),*) },
    };

    let formatted_params = if is_variadic {
        quote! { (*) }
    } else {
        let formatted_params = params.into_iter().map(BuiltinParam::into_token_stream);
        quote! { (#(#formatted_params),*) }
    };

    let return_type = return_type
        .map(|ty| {
            let ty = ty.into_token_stream();
            quote! { -> #ty }
        })
        .unwrap_or_default();

    let fn_with_doc = {
        // TODO: Move this block into a helper function.
        let id_doc = format!(
            "**Static ID:** `{}`",
            microcad_hash::HashId::compile_time_hash(
                format!("__mu::{mod_name}::{fn_name}").as_str()
            )
        );
        let fn_vis = &input_fn.vis;
        let fn_sig = &input_fn.sig;
        let fn_block = &input_fn.block;
        let fn_attrs = &input_fn.attrs;

        quote! {
            #(#fn_attrs)*
            #[doc = ""]
            #[doc = #id_doc]
            #fn_vis #fn_sig {
                #fn_block
            }
        }
    };

    let doc_comment = helpers::attr_fetch_doc(&input_fn.attrs);

    let fn_type = if is_impure {
        quote! { microcad_lang_types::function_type!(#formatted_params #return_type).impure() }
    } else {
        quote! { microcad_lang_types::function_type!(#formatted_params #return_type) }
    };

    quote! {
        #fn_with_doc

        pub static #static_name: BuiltinItem = builtin_item!(
            Function
            #doc_comment
            #mod_name::#name(#fn_type) #defaults
        );
    }
    .into()
}
