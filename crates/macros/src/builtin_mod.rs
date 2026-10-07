// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_mod]` attribute macro.

use crate::prelude::*;

pub(crate) fn builtin_mod_impl(item: TokenStream) -> TokenStream {
    let mut item_mod = parse_macro_input!(item as ItemMod);

    let mod_name = item_mod.ident.clone();

    // Pass the module name to `#[builtin_primitive(...)]`, because it cannot know its parent module.
    item_mod
        .content
        .iter_mut()
        .flat_map(|(_, items)| items)
        .filter_map(|item| match item {
            syn::Item::Struct(ItemStruct { attrs, .. }) => Some(attrs),
            _ => None,
        })
        .flatten()
        .filter(|attr| attr.path().is_ident("builtin_primitive"))
        .for_each(|attr| {
            if let syn::Meta::List(list) = &mut attr.meta {
                let tokens = &list.tokens;
                list.tokens = quote! { #tokens, mod = #mod_name };
            }
        });

    let mod_name = &mod_name;
    let mod_upper_name = helpers::ident_upper(mod_name);

    let collected_builtins: Vec<_> = item_mod
        .content
        .iter()
        .flat_map(|(_, items)| items)
        .filter_map(|item| match item {
            syn::Item::Fn(ItemFn { attrs, sig, .. })
                if helpers::attr_exists(attrs, "builtin_fn") =>
            {
                Some(format_ident!("{}", sig.ident.to_string().to_uppercase()))
            }
            syn::Item::Static(ItemStatic { attrs, ident, .. })
                if helpers::attr_exists(attrs, "builtin_constant") =>
            {
                Some(ident.clone())
            }
            _ => None,
        })
        .collect();

    let doc = helpers::attr_fetch_doc(&item_mod.attrs);

    quote! {
        #item_mod

        pub static #mod_upper_name: BuiltinItem = builtin_item!(Module #doc #mod_name [#(&#mod_name::#collected_builtins),*]);
    }
    .into()
}
