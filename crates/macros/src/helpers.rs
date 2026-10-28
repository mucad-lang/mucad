// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Proc macro helpers. These functions can be used in proc macros.

use crate::prelude::*;

/// A built-in type.
pub(crate) struct BuiltinType(syn::Ident);

impl Parse for BuiltinType {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl From<syn::Ident> for BuiltinType {
    fn from(ident: syn::Ident) -> Self {
        Self(ident)
    }
}

impl quote::ToTokens for BuiltinType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ty = &self.0;

        let expanded = match ty.to_string().as_str() {
            "Scalar" => quote! { mucad_lang_types::Type::scalar() },
            "Angle" => quote! { mucad_lang_types::Type::angle() },
            "Length" => quote! { mucad_lang_types::Type::length() },
            "Color" => quote! { mucad_lang_types::Type::color() },
            "bool" => quote! { mucad_lang_types::Type::Bool },
            "List" => {
                quote! { mucad_lang_types::Type::list() }
            }
            "Mat3" => quote! { mucad_lang_types::Type::matrix(3,3) },
            "Model" => {
                quote! { mucad_lang_types::Type::model() }
            }
            "Model2D" => {
                quote! { mucad_lang_types::Type::model_2d() }
            }
            "Model3D" => {
                quote! { mucad_lang_types::Type::model_3d() }
            }
            _ => quote! { mucad_lang_types::Type::#ty },
        };

        // Append the expanded tokens into the mutable buffer
        expanded.to_tokens(tokens);
    }
}

// Represents a parameter in the macro attribute, e.g.: `lhs: Any`
pub(crate) struct BuiltinParam {
    name: Ident,
    ty: BuiltinType,
    default: Option<syn::Expr>,
}

impl BuiltinParam {
    /// The default value of the parameter, e.g. `message = "Assertion failed"`.
    pub(crate) fn default_tokens(&self) -> Option<TokenStream2> {
        let name = &self.name;
        self.default
            .as_ref()
            .map(|default| quote! { #name = #default })
    }
}

/// The parameter type, e.g. `message: String`, the default value is given separately.
impl quote::ToTokens for BuiltinParam {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let ty = &self.ty;
        quote! { #name: #ty }.to_tokens(tokens);
    }
}

impl Parse for BuiltinParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: BuiltinType = input.parse()?;
        Ok(match input.peek(Token![=]) {
            true => Self {
                name,
                ty,
                default: {
                    input.parse::<Token![=]>()?;
                    Some(input.parse()?)
                },
            },
            false => Self {
                name,
                ty,
                default: None,
            },
        })
    }
}

/// Generate built-in parameters from struct fields.
///
/// Returns the field names and the parameters, e.g. `radius: Type::length()`.
pub(crate) fn struct_params(fields: &Fields) -> Result<(Vec<&Ident>, Vec<TokenStream2>)> {
    let fields = match fields {
        Fields::Named(fields) => &fields.named,
        Fields::Unit => return Ok(Default::default()),
        Fields::Unnamed(fields) => {
            return Err(syn::Error::new_spanned(
                fields,
                "built-in structs require named fields",
            ));
        }
    };

    let mut field_names = Vec::new();
    let mut params = Vec::new();
    for field in fields {
        let name = field.ident.as_ref().expect("named field");
        let syn::Type::Path(type_path) = &field.ty else {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "unsupported built-in field type",
            ));
        };
        let ty = BuiltinType::from(
            type_path
                .path
                .segments
                .last()
                .expect("type path segment")
                .ident
                .clone(),
        );

        params.push(quote! { #name: #ty });
        field_names.push(name);
    }

    Ok((field_names, params))
}

/// Extract doc comment string from attributes.
pub fn attr_fetch_doc(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc")
                && let Ok(nv) = attr.meta.require_name_value()
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &nv.value
            {
                Some(lit_str.value().trim().to_string())
            } else {
                None
            }
        })
        .reduce(|acc, line| format!("{acc}\n{line}"))
        .unwrap_or_default()
}

/// Check if an attribute with a name exists with an attribute list.
pub fn attr_exists(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

/// Generate the static ID doc comment of a built-in, e.g. for `__mu::ops::extrude`.
pub fn static_id_doc(mod_name: &syn::Ident, name: &syn::Ident) -> TokenStream2 {
    let id_doc = format!(
        "**Static ID:** `{}`",
        mucad_hash::HashId::compile_time_hash(format!("__mu::{mod_name}::{name}").as_str())
    );
    quote! {
        #[doc = ""]
        #[doc = #id_doc]
    }
}

/// Parses `mod = #mod_name`, which is injected by `#[builtin_mod]`.
pub(crate) fn parse_mod_name(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![mod]>()?;
    input.parse::<Token![=]>()?;
    input.parse()
}

/// Error for built-in structs used outside of `#[builtin_mod]`.
pub(crate) fn missing_mod_error(macro_name: &str, span: &impl quote::ToTokens) -> TokenStream {
    syn::Error::new_spanned(
        span,
        format!("#[{macro_name}] must be used inside a #[builtin_mod] module"),
    )
    .to_compile_error()
    .into()
}

/// A function declaration without body, e.g.:
///
/// ```rs
/// pub fn extrude(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;
/// ```
pub(crate) struct BodylessFn {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    sig: syn::Signature,
}

impl Parse for BodylessFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let sig: syn::Signature = input.parse()?;
        if input.peek(syn::token::Brace) {
            return Err(syn::Error::new_spanned(
                &sig,
                "the function body is generated, remove it and end the declaration with `;`",
            ));
        }
        input.parse::<Token![;]>()?;
        Ok(Self { attrs, vis, sig })
    }
}

/// Complete a bodyless function by forwarding to `BuiltinOperationCall::call`
/// of the struct named after the function (e.g. `extrude` -> `Extrude`).
pub(crate) fn expand_operation_call_fn(
    macro_name: &str,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("#[{macro_name}] on functions takes no arguments"),
        )
        .to_compile_error()
        .into();
    }

    let BodylessFn { attrs, vis, sig } = parse_macro_input!(item as BodylessFn);

    let args = match sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(syn::PatType { pat, .. }) => match pat.as_ref() {
                syn::Pat::Ident(pat_ident) => Ok(&pat_ident.ident),
                pat => Err(syn::Error::new_spanned(pat, "expected a parameter name")),
            },
            syn::FnArg::Receiver(receiver) => Err(syn::Error::new_spanned(
                receiver,
                "unexpected `self` parameter",
            )),
        })
        .collect::<Result<Vec<_>>>()
    {
        Ok(args) => args,
        Err(err) => return err.to_compile_error().into(),
    };

    let struct_name = ident_pascal(&sig.ident);

    quote! {
        #(#attrs)*
        #vis #sig {
            <#struct_name as crate::BuiltinOperationCall>::call(#(#args),*)
        }
    }
    .into()
}

/// Generate pascal case name (e.g., `linear_extrude` -> `LinearExtrude`)
pub fn ident_pascal(ident: &syn::Ident) -> syn::Ident {
    let name: String = ident
        .to_string()
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                .unwrap_or_default()
        })
        .collect();
    quote::format_ident!("{name}", span = ident.span())
}

/// Generate snake case name (e.g., `LinearExtrude` -> `linear_extrude`)
pub fn ident_snake(ident: &syn::Ident) -> syn::Ident {
    let mut name = String::new();
    for (i, c) in ident.to_string().chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                name.push('_');
            }
            name.extend(c.to_lowercase());
        } else {
            name.push(c);
        }
    }
    quote::format_ident!("{name}")
}

/// Generate uppercase name (e.g., `add` -> `ADD`)
pub fn ident_upper(ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("{}", ident.to_string().to_uppercase())
}
