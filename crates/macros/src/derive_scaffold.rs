// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[derive(Scaffold)]` derive macro.
//!
//! This is used for building a tree of IR items (aka "scaffolding").

use crate::prelude::*;

pub fn derive_scaffold_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Ensure we are deriving for a struct with named fields
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("#[derive(Scaffold)] only supports structs with named fields"),
        },
        _ => panic!("#[derive(Scaffold)] only supports structs"),
    };

    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    quote! {
        impl crate::Scaffold for #name {
            fn scaffold(self, context: &mut crate::LowerContext) -> ir::NodeId {
                #(
                    self.#field_idents.scaffold(context);
                )*
                *context.top_node()
            }
        }
    }
    .into()
}
