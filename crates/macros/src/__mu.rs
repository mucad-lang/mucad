// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use proc_macro2::Span;

use crate::prelude::*;

struct MuInput {
    path: Path,
}

impl Parse for MuInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MuInput {
            path: input.parse()?,
        })
    }
}

/// Path to the `microcad-builtin` crate root.
fn get_builtin_crate() -> syn::Result<TokenStream2> {
    match proc_macro_crate::crate_name("microcad-builtin") {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(quote! { ::#ident })
        }
        Err(err) => Err(syn::Error::new(
            Span::call_site(),
            format!("`__mu!` requires the crate `microcad-builtin`: {err}"),
        )),
    }
}

pub(crate) fn __mu_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MuInput);
    let path = &input.path;

    if path.segments.len() != 2 {
        return syn::Error::new_spanned(
            path,
            "Please provide a path with exactly two segments (e.g. `core::add`).",
        )
        .to_compile_error()
        .into();
    }

    let builtin_crate = match get_builtin_crate() {
        Ok(builtin_crate) => builtin_crate,
        Err(err) => return err.to_compile_error().into(),
    };
    let module = &path.segments[0].ident; // `core`
    let item = &path.segments[1].ident; // `add`
    let name = format!("__mu::{module}::{item}"); // `__mu::core::add`

    quote! {
        {
            // 1. Compile-time existence check for the built-in item.
            #[allow(unused_imports)]
            use #builtin_crate::mu::#module::#item as _;

            // 2. The ID is the hash of the full name, like in `builtin_info!`.
            const ID: #builtin_crate::BuiltinId = #builtin_crate::BuiltinId::from_name(#name);
            ID
        }
    }
    .into()
}
