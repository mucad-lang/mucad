// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation for the `#[builtin_boolean_op]` attribute macro.
//!
//! The macro is used in pairs, a struct and a bodyless function:
//!
//! ```rs
//! /// Subtract all following child models from the first one.
//! #[builtin_boolean_op]
//! pub struct Subtract;
//!
//! impl From<Subtract> for BooleanOp {
//!     fn from(_: Subtract) -> Self {
//!         BooleanOp::Difference
//!     }
//! }
//!
//! /// Subtract all following child models from the first one.
//! #[builtin_boolean_op]
//! pub fn subtract(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;
//! ```
//!
//! The struct implements `Builtin` and `BuiltinOperationCall`. The function is named after the
//! struct in snake case and its body calls `BuiltinOperationCall::call`.

use crate::prelude::*;

struct BuiltinBooleanOpSig {
    mod_name: Option<Ident>,
}

impl Parse for BuiltinBooleanOpSig {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            mod_name: match input.is_empty() {
                true => None,
                false => Some(helpers::parse_mod_name(input)?),
            },
        })
    }
}

pub(crate) fn builtin_boolean_op_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ItemStruct>(item.clone()) {
        Ok(input_struct) => builtin_boolean_op_struct(attr, input_struct),
        Err(_) => helpers::expand_operation_call_fn("builtin_boolean_op", attr, item),
    }
}

fn builtin_boolean_op_struct(attr: TokenStream, input_struct: ItemStruct) -> TokenStream {
    let BuiltinBooleanOpSig { mod_name } = parse_macro_input!(attr as BuiltinBooleanOpSig);

    let struct_name = &input_struct.ident;

    let Some(mod_name) = mod_name else {
        return helpers::missing_mod_error("builtin_boolean_op", struct_name);
    };

    let (field_names, params) = match helpers::struct_params(&input_struct.fields) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    let fn_name = helpers::ident_snake(struct_name);
    let static_name = helpers::ident_upper(&fn_name);
    let doc_comment = helpers::attr_fetch_doc(&input_struct.attrs);
    let id_doc = helpers::static_id_doc(&mod_name, &fn_name);

    quote! {
        #id_doc
        #input_struct

        pub static #static_name: &crate::BuiltinItem = <#struct_name as crate::Builtin>::ITEM;

        impl crate::Builtin for #struct_name {
            const ITEM: &'static crate::BuiltinItem = &crate::BuiltinItem::Operation(crate::BuiltinOperation::new(
                crate::builtin_info!(#doc_comment #mod_name::#fn_name),
                || microcad_lang_types::function_type!((
                    self: microcad_lang_types::Type::model()
                    #(, #params)*
                ) -> microcad_lang_types::Type::model()),
                #fn_name,
            ));
        }

        impl crate::BuiltinOperationCall for #struct_name {
            fn call(
                args: microcad_lang_types::Arguments,
                _ctx: &mut crate::BuiltinEvalContext,
            ) -> Result<microcad_lang_types::ModelTree, crate::BuiltinError> {
                use microcad_lang_types::*;
                let self_: std::rc::Rc<ModelTree> = args.try_get("self")?;
                let op = model::BooleanOp::from(Self {
                    #(#field_names: args.try_get(stringify!(#field_names))?,)*
                });

                // The operands are the children of a group, like `ModelTree::apply_boolean_op` builds.
                let mut operands = std::rc::Rc::unwrap_or_clone(self_);
                if !matches!(operands.root().element, microcad_lang_types::model::Element::Group) {
                    let mut group = ModelTree::new(
                        model::Element::Group,
                    );
                    group.append(operands);
                    operands = group;
                }

                let mut tree = microcad_lang_types::ModelTree::new(
                    Model::from(op).with_op_properties(args),
                );
                tree.append(operands);
                Ok(tree)
            }
        }
    }
    .into()
}
