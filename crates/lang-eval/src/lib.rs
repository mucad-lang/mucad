// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evaluation of symbols.

mod argument_match;

mod builtin;
mod context;
mod eval_error;
mod function;
mod issue;
mod source;
mod symbol;
mod workbench;

pub use argument_match::*;
pub use issue::*;

pub use context::EvalContext;

use microcad_lang_types::{ArgumentValueList, ModelTree, Value};

/// Evaluation trait.
///
/// The return type `T` defines to which output type the type is evaluated.
pub trait Eval<T = Value> {
    /// Evaluate a syntax element into a type `T`.
    fn eval(&self, context: &mut EvalContext) -> EvalResult<T>;
}

impl<T> Eval<ModelTree> for T
where
    T: Eval,
{
    fn eval(&self, context: &mut EvalContext) -> EvalResult<ModelTree> {
        let value: Value = self.eval(context)?;
        Ok(value.into())
    }
}

/// Trait for calls with argument list.
pub trait Callable<T = Value> {
    /// Evaluate call into value (if possible).
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult<T>;
}

impl<T> Callable<ModelTree> for T
where
    T: Callable,
{
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult<ModelTree> {
        let value: Value = self.call(args, context)?;
        Ok(value.into())
    }
}
