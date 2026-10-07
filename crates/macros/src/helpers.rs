// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Proc macro helpers. These functions can be used in proc macros.

use crate::prelude::*;

/// A built-in type.
pub(crate) struct BuiltinType(syn::Ident);

impl Parse for BuiltinType {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl From<syn::Ident> for BuiltinType {
    fn from(ident: syn::Ident) -> Self {
        Self(ident)
    }
}

impl quote::ToTokens for BuiltinType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ty = &self.0;

        let expanded = match ty.to_string().as_str() {
            "Scalar" => quote! { microcad_lang_types::Type::scalar() },
            "Angle" => quote! { microcad_lang_types::Type::angle() },
            "Length" => quote! { microcad_lang_types::Type::length() },
            "Color" => quote! { microcad_lang_types::Type::color() },
            "List" => {
                quote! { microcad_lang_types::Type::list() }
            }
            "Mat3" => quote! { microcad_lang_types::Type::matrix(3,3) },
            "Model" => {
                quote! { microcad_lang_types::Type::model() }
            }
            "Model2D" => {
                quote! { microcad_lang_types::Type::model_2d() }
            }
            "Model3D" => {
                quote! { microcad_lang_types::Type::model_3d() }
            }
            _ => quote! { microcad_lang_types::Type::#ty },
        };

        // Append the expanded tokens into the mutable buffer
        expanded.to_tokens(tokens);
    }
}

// Represents a parameter in the macro attribute, e.g.: `lhs: Any`
pub(crate) struct BuiltinParam {
    name: Ident,
    ty: BuiltinType,
    default: Option<syn::Expr>,
}

impl quote::ToTokens for BuiltinParam {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let ty = &self.ty;
        match &self.default {
            Some(default) => {
                quote! {
                    #name: #ty = #default
                }
            }
            None => {
                quote! {
                    #name: #ty
                }
            }
        }
        .to_tokens(tokens);
    }
}

impl Parse for BuiltinParam {
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

/// Extract doc comment string from attributes.
pub fn attr_fetch_doc(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc")
                && let Ok(nv) = attr.meta.require_name_value()
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &nv.value
            {
                Some(lit_str.value().trim().to_string())
            } else {
                None
            }
        })
        .reduce(|acc, line| format!("{acc}\n{line}"))
        .unwrap_or_default()
}

/// Check if an attribute with a name exists with an attribute list.
pub fn attr_exists(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

/// Generate uppercase name (e.g., `add` -> `ADD`)
pub fn ident_upper(ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("{}", ident.to_string().to_uppercase())
}
