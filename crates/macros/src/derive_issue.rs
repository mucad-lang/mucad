// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `#[derive(Issue)]` implementation.

use crate::prelude::*;

/// The value of `#[issue(key = value)]`, e.g. `code = "render"` or `diagnostic = kind`.
fn issue_attr<T: Parse>(attrs: &[syn::Attribute], key: &str) -> Result<Option<T>> {
    let mut value = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("issue")) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident(key) {
                return Err(meta.error(format!("expected `{key} = ...`")));
            }
            value = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }
    Ok(value)
}

/// Derives `Issue` and `miette::Diagnostic` for an enum with the variants `Error`, `Warning` and
/// `Advice`.
///
/// The code of a diagnostic is the code of the error, warning or advice, or the code prefix
/// with `::error`, `::warning` or `::advice`, e.g. `render::error` for `#[issue(code = "render")]`.
/// Help, labels and related diagnostics are those of the error, warning or advice, which can be
/// returned by a method, e.g. `#[issue(diagnostic = kind)]` for `EvalError::kind()`.
pub fn derive_issue_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_issue(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_issue(input: &DeriveInput) -> Result<TokenStream2> {
    let name = &input.ident;
    let error = |message: String| syn::Error::new_spanned(name, message);
    let Data::Enum(data) = &input.data else {
        return Err(error("#[derive(Issue)] requires an enum".into()));
    };
    let code: syn::LitStr = issue_attr(&input.attrs, "code")?
        .ok_or_else(|| error("expected `#[issue(code = \"...\")]`".into()))?;

    let (mut variants, mut types, mut diagnostics, mut codes) = (vec![], vec![], vec![], vec![]);
    // The variants are named after their `miette::Severity`.
    for variant in ["Error", "Warning", "Advice"] {
        let found = data
            .variants
            .iter()
            .find(|v| v.ident == variant)
            .ok_or_else(|| error(format!("expected a variant `{variant}`")))?;
        let Fields::Unnamed(fields) = &found.fields else {
            return Err(error(format!("expected `{variant}` with a single field")));
        };
        let [field] = fields.unnamed.iter().collect::<Vec<_>>()[..] else {
            return Err(error(format!("expected `{variant}` with a single field")));
        };

        variants.push(&found.ident);
        types.push(&field.ty);
        diagnostics.push(match issue_attr::<Ident>(&found.attrs, "diagnostic")? {
            Some(method) => quote! { inner.#method() },
            None => quote! { inner },
        });
        codes.push(format!("{}::{}", code.value(), variant.to_lowercase()));
    }
    let [error, warning, advice] = &types[..] else {
        unreachable!("three variants")
    };

    Ok(quote! {
        const _: () = {
            use mucad_lang_base::Issue;
            use miette::{Diagnostic, LabeledSpan, Severity, SourceCode};

            impl Issue for #name {
                type Error = #error;
                type Warning = #warning;
                type Advice = #advice;

                fn severity(&self) -> Severity {
                    match self { #(Self::#variants(_) => Severity::#variants,)* }
                }

                fn diagnostic(&self) -> &dyn Diagnostic {
                    match self { #(Self::#variants(inner) => #diagnostics,)* }
                }
            }

            impl Diagnostic for #name {
                fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                    self.diagnostic().code().or_else(|| match self {
                        #(Self::#variants(_) => Some(Box::new(#codes)),)*
                    })
                }

                fn severity(&self) -> Option<Severity> {
                    Some(Issue::severity(self))
                }

                fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                    self.diagnostic().help()
                }

                fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                    self.diagnostic().url()
                }

                fn source_code(&self) -> Option<&dyn SourceCode> {
                    self.diagnostic().source_code()
                }

                fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
                    self.diagnostic().labels()
                }

                fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
                    self.diagnostic().related()
                }

                fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
                    self.diagnostic().diagnostic_source()
                }
            }
        };
    })
}
