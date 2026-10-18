// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[derive(Visit)]` derive macro.
//!
//! This is used implementing AST visitors.

use crate::prelude::*;

pub fn derive_visit_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    fn has_visit_default(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path().is_ident("visit")
                && attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") {
                            Ok(())
                        } else {
                            Err(meta.error("unsupported container visit attribute"))
                        }
                    })
                    .is_ok()
        })
    }

    // Check if the container itself has the #[visit(default)] attribute
    if has_visit_default(&input.attrs) {
        return quote! {
            impl crate::ast::visitor::Visit for #name {
                fn visit<'ast, V>(&'ast self, _visitor: &mut V) -> std::ops::ControlFlow<V::BreakTy>
                    where V: crate::ast::visitor::Visitor<'ast> + ?Sized
                {
                    std::ops::ControlFlow::Continue(())
                }
            }
        }
        .into();
    }

    fn has_visit_skip(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            // Check if the attribute path is exactly "visit"
            attr.path().is_ident("visit") &&
                // Parse inside the parentheses: #[visit(...)]
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        Ok(())
                    } else {
                        Err(meta.error("unsupported visit attribute"))
                    }
                }).is_ok()
        })
    }

    match input.data {
        Data::Struct(s) => match s.fields {
            // Named struct
            Fields::Named(f) => {
                // Generate code for each field
                let recurse = f.named.iter().map(|f| {
                    if !has_visit_skip(&f.attrs) {
                        let name = &f.ident;
                        // This assumes every field in your AST implements Visit
                        quote! {
                            self.#name.visit(visitor)?;
                        }
                    } else {
                        proc_macro2::TokenStream::new()
                    }
                });

                quote! {
                    impl crate::ast::visitor::Visit for #name {
                        fn visit<'ast, V>(&'ast self, visitor: &mut V) -> std::ops::ControlFlow<V::BreakTy>
                            where V: crate::ast::visitor::Visitor<'ast> + ?Sized
                        {
                            #(#recurse)*
                            std::ops::ControlFlow::Continue(())
                        }
                    }
                }
                .into()
            }
            _ => panic!("Only named fields supported"),
        },
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                // Scenario A: The entire variant is marked #[skip_visit]
                if has_visit_skip(&variant.attrs) {
                    return match &variant.fields {
                        Fields::Named(_) => quote! { #name::#variant_name { .. } => {} },
                        Fields::Unnamed(_) => quote! { #name::#variant_name(..) => {} },
                        Fields::Unit => quote! { #name::#variant_name => {} },
                    };
                }

                // Scenario B: Individual fields inside the variant might be marked #[skip_visit]
                match &variant.fields {
                    Fields::Named(fields) => {
                        // Create pattern bindings for ALL fields, but only generate visit calls for non-skipped ones
                        let idents = fields.named.iter().map(|f| &f.ident);
                        let visit_calls = fields
                            .named
                            .iter()
                            .filter(|f| !has_visit_skip(&f.attrs))
                            .map(|f| {
                                let name = &f.ident;
                                quote! { #name.visit(visitor)?; }
                            });

                        quote! {
                            #name::#variant_name { #(#idents),* } => {
                                #(#visit_calls)*
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        // Dynamically create temporary names for destructuring
                        let idents = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| quote::format_ident!("field{}", i))
                            .collect::<Vec<_>>();

                        let visit_calls = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .filter(|(_, f)| !has_visit_skip(&f.attrs))
                            .map(|(i, _)| {
                                let field_ident = quote::format_ident!("field{}", i);
                                quote! { #field_ident.visit(visitor)?; }
                            });

                        quote! {
                            #name::#variant_name(#(#idents),*) => {
                                #(#visit_calls)*
                            }
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            #name::#variant_name => {}
                        }
                    }
                }
            });

            quote! {
                impl crate::ast::visitor::Visit for #name {
                    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> std::ops::ControlFlow<V::BreakTy>
                        where V: crate::ast::visitor::Visitor<'ast> + ?Sized
                    {
                        match self {
                            #(#variants)*
                        }
                        std::ops::ControlFlow::Continue(())
                    }
                }
            }
            .into()
        }
        _ => panic!("Only structs supported"),
    }
}
