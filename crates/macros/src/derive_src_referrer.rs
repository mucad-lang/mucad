// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `#[derive(SrcReferrer)]` implementation.

use crate::prelude::*;

/// Derives the `SrcReferrer` trait for structs.
///
/// Automatically implements `src_ref()`  for a struct field named `src_ref`.
///
/// # Panics
///
/// Will fail to compile if applied to tuple structs, enums, unions, or unit structs.
pub fn derive_src_referrer_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let has_src_ref = match &input.data {
        Data::Struct(syn::DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields
            .named
            .iter()
            .any(|f| f.ident.as_ref().is_some_and(|i| i == "src_ref")),
        _ => false,
    };

    if !has_src_ref {
        return syn::Error::new_spanned(
            name,
            "#[derive(SrcReferrer)] requires a struct with a named `src_ref` field.",
        )
        .to_compile_error()
        .into();
    }

    quote! {
        impl mucad_lang_base::SrcReferrer for #name {
            fn src_ref(&self) -> mucad_lang_base::SrcRef {
                self.src_ref
            }
        }
    }
    .into()
}
