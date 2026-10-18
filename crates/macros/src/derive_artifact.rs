// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[derive(Artifact)]` derive macro.
//!
//! This can be used to annotate structs compiler artifacts that can be serialized or deserialized.

use crate::prelude::*;

pub fn derive_artifact_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    quote! {
        impl mucad_lang_base::Artifact for #name {
            fn kind() -> mucad_lang_base::ArtifactKind {
                mucad_lang_base::ArtifactKind::#name
            }
        }
    }
    .into()
}
