// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_constant(...)` attribute macro.

use crate::prelude::*;

// Parses the syntax inside #[builtin_fn(#mod_name::#name)]
struct BuiltinConstantSig {
    /// E.g. `math`
    mod_name: Ident,
    /// E:g. `PI`
    name: Ident,
}

impl Parse for BuiltinConstantSig {
    fn parse(input: ParseStream) -> Result<Self> {
        // 1. Parse `mod_name::name`
        let mod_name: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let name: Ident = input.parse()?;

        Ok(Self { mod_name, name })
    }
}

pub(crate) fn builtin_constant_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the macro attribute syntax
    let BuiltinConstantSig { name, mod_name } = parse_macro_input!(attr as BuiltinConstantSig);

    // Parse the annotated static item (e.g., `pub static PI: Builtin = std::f64::consts::PI;`)
    let input_static = parse_macro_input!(item as ItemStatic);
    let static_name = &input_static.ident;

    if name != *static_name {
        return syn::Error::new_spanned(
            &name,
            format!(
                "builtin constant name name `{}` does not match Rust static name `{}`",
                name, static_name
            ),
        )
        .to_compile_error()
        .into();
    }

    let doc_comment = helpers::attr_fetch_doc(&input_static.attrs);
    let expr = &input_static.expr;

    quote! {
        pub static #static_name: BuiltinItem = builtin_item!(
            Constant
            #doc_comment
            #mod_name::#name = #expr
        );
    }
    .into()
}
