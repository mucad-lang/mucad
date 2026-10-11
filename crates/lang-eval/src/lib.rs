// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evaluation of symbols.

mod argument_match;

mod builtin;
mod context;
mod function;
mod issue;
mod source;
mod symbol;
mod workbench;

pub use argument_match::*;
pub use issue::*;

pub use context::{EvalContext, EvalContextBuilder};

use microcad_lang_base::{IssueResult, PushIssue};
use microcad_lang_types::{ArgumentValueList, ModelTree, Value};

/// The eval compiler stage: evaluate the library of the context into a model tree.
///
/// Recoverable errors are returned with the model, see [`IssueResult`].
pub fn eval(mut context: EvalContext) -> IssueResult<ModelTree, EvalIssue> {
    match context.eval_library() {
        Ok(value) => Ok((ModelTree::from(value), context.issues())),
        Err(err) => {
            let mut issues = context.issues();
            issues.push_err(err);
            Err(issues)
        }
    }
}

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
