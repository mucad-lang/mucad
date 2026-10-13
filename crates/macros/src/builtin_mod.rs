// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_mod]` attribute macro.

use crate::prelude::*;

/// Attribute macros on structs which need the module name.
const BUILTIN_STRUCT_ATTRS: [&str; 4] = [
    "builtin_primitive",
    "builtin_op",
    "builtin_transform",
    "builtin_boolean_op",
];

pub(crate) fn builtin_mod_impl(item: TokenStream) -> TokenStream {
    let mut item_mod = parse_macro_input!(item as ItemMod);

    let mod_name = item_mod.ident.clone();

    // Pass the module name to built-in structs (e.g. `#[builtin_primitive(...)]`),
    // because they cannot know their parent module.
    item_mod
        .content
        .iter_mut()
        .flat_map(|(_, items)| items)
        .filter_map(|item| match item {
            syn::Item::Struct(ItemStruct { attrs, .. }) => Some(attrs),
            _ => None,
        })
        .flatten()
        .filter(|attr| {
            BUILTIN_STRUCT_ATTRS
                .iter()
                .any(|name| attr.path().is_ident(name))
        })
        .for_each(|attr| match &mut attr.meta {
            syn::Meta::List(list) if list.tokens.is_empty() => {
                list.tokens = quote! { mod = #mod_name };
            }
            syn::Meta::List(list) => {
                let tokens = &list.tokens;
                list.tokens = quote! { #tokens, mod = #mod_name };
            }
            // `#[builtin_transform]` or `#[builtin_boolean_op]` without arguments.
            meta @ syn::Meta::Path(_) => {
                let path = meta.path().clone();
                *meta = syn::parse_quote! { #path(mod = #mod_name) };
            }
            syn::Meta::NameValue(_) => {}
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
                let ident = helpers::ident_upper(&sig.ident);
                Some(quote! { &#mod_name::#ident })
            }
            syn::Item::Static(ItemStatic { attrs, ident, .. })
                if helpers::attr_exists(attrs, "builtin_constant") =>
            {
                Some(quote! { &#mod_name::#ident })
            }
            // Primitive statics are already references: `pub static CIRCLE: &BuiltinItem`.
            syn::Item::Struct(ItemStruct { attrs, ident, .. })
                if helpers::attr_exists(attrs, "builtin_primitive") =>
            {
                let ident = helpers::ident_upper(ident);
                Some(quote! { #mod_name::#ident })
            }
            // Operation, transform and boolean operation statics are references, too,
            // named after their function.
            syn::Item::Struct(ItemStruct { attrs, ident, .. })
                if helpers::attr_exists(attrs, "builtin_op")
                    || helpers::attr_exists(attrs, "builtin_transform")
                    || helpers::attr_exists(attrs, "builtin_boolean_op") =>
            {
                let ident = helpers::ident_upper(&helpers::ident_snake(ident));
                Some(quote! { #mod_name::#ident })
            }
            _ => None,
        })
        .collect();

    let doc = helpers::attr_fetch_doc(&item_mod.attrs);

    quote! {
        #item_mod

        pub static #mod_upper_name: BuiltinItem = builtin_item!(Module #doc #mod_name [#(#collected_builtins),*]);
    }
    .into()
}
