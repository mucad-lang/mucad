// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate proc_macro;

mod __mu;
mod builtin_constant;
mod builtin_fn;
mod builtin_mod;
mod builtin_op;
mod builtin_primitive;
mod builtin_transform;
mod derive_artifact;
mod derive_identifiable;
mod derive_scaffold;
mod derive_src_referrer;
mod derive_visit;
mod helpers;
mod include_inner_docs;
mod parameter_list;
mod test_builtin_fn;

pub(crate) mod prelude {
    pub use proc_macro::TokenStream;
    pub use proc_macro2::TokenStream as TokenStream2;

    pub use quote::quote;
    pub use syn::parse::{Parse, ParseStream};
    pub use syn::spanned::Spanned;
    pub use syn::{
        Data, DeriveInput, Expr, Fields, Ident, ItemFn, ItemMod, ItemStatic, ItemStruct, Path,
        Result, Token, parse_macro_input,
    };

    pub(crate) use super::helpers;
}

use prelude::*;

/// Macro to get the hash from built-in function.
#[proc_macro]
pub fn __mu(input: TokenStream) -> TokenStream {
    __mu::__mu_impl(input)
}

#[proc_macro_attribute]
pub fn builtin_constant(attr: TokenStream, item: TokenStream) -> TokenStream {
    builtin_constant::builtin_constant_impl(attr, item)
}

#[proc_macro_attribute]
pub fn builtin_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    builtin_fn::builtin_fn_impl(attr, item)
}

#[proc_macro_attribute]
pub fn builtin_mod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    builtin_mod::builtin_mod_impl(item)
}

/// Declare a built-in operation with a struct and a bodyless function.
///
/// On a struct, the argument is the model type of `self` and the result, e.g.
/// `#[builtin_op(Geometry2D -> Geometry3D)]`. Implements `Builtin`, `BuiltinConstruct` and
/// `BuiltinOperationCall` for the struct and generates a static, e.g.
/// `pub static EXTRUDE: &BuiltinItem = Extrude::ITEM;`.
///
/// On a function, `#[builtin_op]` has no arguments. The function is named after the struct in
/// snake case (e.g. `Extrude` -> `extrude`) and its generated body calls
/// `BuiltinOperationCall::call`.
///
/// Must be used inside a `#[builtin_mod]` module, which provides the module name.
#[proc_macro_attribute]
pub fn builtin_op(attr: TokenStream, item: TokenStream) -> TokenStream {
    builtin_op::builtin_op_impl(attr, item)
}

/// Declare a built-in affine transform with a struct and a bodyless function.
///
/// On a struct, `#[builtin_transform]` has no arguments and the struct must implement
/// `Into<AffineTransform>`. Implements `Builtin` and `BuiltinOperationCall` for the struct
/// and generates a static, e.g. `pub static TRANSLATE: &BuiltinItem = Translate::ITEM;`.
///
/// On a function, `#[builtin_transform]` has no arguments. The function is named after the
/// struct in snake case (e.g. `Translate` -> `translate`) and its generated body calls
/// `BuiltinOperationCall::call`.
///
/// Must be used inside a `#[builtin_mod]` module, which provides the module name.
#[proc_macro_attribute]
pub fn builtin_transform(attr: TokenStream, item: TokenStream) -> TokenStream {
    builtin_transform::builtin_transform_impl(attr, item)
}

/// Declare a struct as built-in primitive.
///
/// Implements `Builtin`, `BuiltinConstruct` and `BuiltinPrimitiveCall` for the struct and
/// generates a static, e.g. `pub static CIRCLE: &BuiltinItem = Circle::ITEM;`.
///
/// The argument is the output model type, e.g. `#[builtin_primitive(Geometry2D)]`.
/// Must be used inside a `#[builtin_mod]` module, which provides the module name.
#[proc_macro_attribute]
pub fn builtin_primitive(attr: TokenStream, item: TokenStream) -> TokenStream {
    builtin_primitive::builtin_primitive_impl(attr, item)
}

/// Create a test case for a built-in function.
///
/// Expands `core::add(3, 4) == 7` to
/// ```rs
/// fn add() {
///     let mut ctx = BuiltinEvalContext::new();
///     assert_that!(core::add(arguments!(3, 4), &mut ctx), eq(8))
/// }
/// ```
#[proc_macro_attribute]
pub fn test_builtin_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    test_builtin_fn::test_builtin_fn_impl(attr, item)
}

/// Derive the trait `Artifact` for this struct to mark it as compiler artifact.
#[proc_macro_derive(Artifact)]
pub fn derive_artifact(input: TokenStream) -> TokenStream {
    derive_artifact::derive_artifact_impl(input)
}

/// Derive the trait `Identifiable` for this struct to return the `id` field.
#[proc_macro_derive(Identifiable)]
pub fn derive_identifiable(input: TokenStream) -> TokenStream {
    derive_identifiable::derive_identifiable_impl(input)
}

#[proc_macro_derive(Scaffold)]
pub fn derive_scaffold(input: TokenStream) -> TokenStream {
    derive_scaffold::derive_scaffold_impl(input)
}

#[proc_macro_derive(SrcReferrer)]
pub fn derive_src_referrer(input: TokenStream) -> TokenStream {
    derive_src_referrer::derive_src_referrer_impl(input)
}

#[proc_macro_derive(Visit, attributes(visit))]
pub fn derive_visit(input: TokenStream) -> TokenStream {
    derive_visit::derive_visit_impl(input)
}

/// Extract the inner doc comments `//!` in a source file.
#[proc_macro]
pub fn include_inner_docs(input: TokenStream) -> TokenStream {
    include_inner_docs::include_inner_docs_impl(input)
}

/// Generate a parameter list µcad builtin types.
#[proc_macro]
pub fn parameter_list(input: TokenStream) -> TokenStream {
    parameter_list::parameter_list_impl(input)
}
