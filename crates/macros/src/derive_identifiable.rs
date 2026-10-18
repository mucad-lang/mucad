// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `#[derive(Identifiable)]` implementation.

use crate::prelude::*;

/// Derives the `SrcReferrer` trait for structs.
///
/// Automatically implements `trait Identifiable` for a struct field named `id`.
///
/// # Panics
///
/// Will fail to compile if applied to tuple structs, enums, unions, or unit structs.
pub fn derive_identifiable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let has_id = match &input.data {
        Data::Struct(syn::DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields
            .named
            .iter()
            .any(|f| f.ident.as_ref().is_some_and(|i| i == "id")),
        _ => false,
    };

    if !has_id {
        return syn::Error::new_spanned(
            name,
            "#[derive(Identifiable)] requires a struct with a named `id` field.",
        )
        .to_compile_error()
        .into();
    }

    quote! {
        impl mucad_lang_base::Identifiable for #name {
            fn id_ref(&self) -> &mucad_lang_base::Identifier  {
                &self.id
            }
        }
    }
    .into()
}
