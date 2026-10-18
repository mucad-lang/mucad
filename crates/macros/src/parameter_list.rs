// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Proc macro `parameter_list!` to declare µcad like parameter list in Rust.

use quote::ToTokens;

use crate::helpers::BuiltinType;
use crate::prelude::*;

// Represents a parameter in the macro attribute, e.g.: `lhs: Any`
pub(crate) struct Parameter {
    name: Ident,
    ty: BuiltinType,
    default: Option<syn::Expr>,
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: BuiltinType = input.parse()?;
        Ok(match input.peek(Token![=]) {
            true => Self {
                name,
                ty,
                default: {
                    input.parse::<Token![=]>()?;
                    Some(input.parse()?)
                },
            },
            false => Self {
                name,
                ty,
                default: None,
            },
        })
    }
}

impl quote::ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let ty = &self.ty;
        match &self.default {
            Some(default) => {
                quote! {
                    Parameter::new(stringify!(#name), #ty).with_default(#default)
                }
            }
            None => {
                quote! {
                    Parameter::new(stringify!(#name), #ty)
                }
            }
        }
        .to_tokens(tokens);
    }
}

// Parses the syntax inside #[builtin_fn(#mod_name::#name(arg1: Type1, ...) -> #return_type)]
struct ParameterList {
    params: Vec<Parameter>,
}

impl Parse for ParameterList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut params = Vec::new();
        while !input.is_empty() {
            params.push(input.parse::<Parameter>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(Self { params })
    }
}

pub(crate) fn parameter_list_impl(input: TokenStream) -> TokenStream {
    // Parse the macro attribute syntax
    let ParameterList { params } = parse_macro_input!(input as ParameterList);

    let formatted_params = params.into_iter().map(Parameter::into_token_stream);
    quote! { vec![#(#formatted_params),*] }.into()
}
