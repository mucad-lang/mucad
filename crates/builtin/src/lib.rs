// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad built-in crate.

mod diag;
mod item;
pub use diag::{BuiltinAdvice, BuiltinError, BuiltinIssue, BuiltinResult, BuiltinWarning};

pub use item::{
    BuiltinConstant, BuiltinFunction, BuiltinItem, BuiltinModule, BuiltinOperation,
    BuiltinPrimitive,
};

mod library;
pub mod mu;
pub use library::BuiltinLibrary;
use mucad_lang_base::{DisplayWithCtx, IssueList, PushIssue, SrcRef};

use derive_more::Debug;
use mucad_lang_types::{Arguments, Model, ModelTree, Value, model};

pub use mucad_lang_base::{BuiltinId, BuiltinInfo, builtin_info};
pub use mucad_macros::__mu;

#[derive(Debug, Default)]
pub struct BuiltinEvalContext<'a> {
    /// Current function being evaluated.
    pub current_fn: Option<&'a BuiltinFunction>,

    /// Source code reference of the call, e.g. for the location of errors.
    pub src_ref: SrcRef,

    /// Diagnostics.
    pub issues: IssueList<BuiltinIssue>,
}

/// A context for a call at `src_ref`, e.g. errors refer to the call.
impl From<SrcRef> for BuiltinEvalContext<'_> {
    fn from(src_ref: SrcRef) -> Self {
        Self {
            src_ref,
            ..Default::default()
        }
    }
}

impl<'a> PushIssue<BuiltinIssue> for BuiltinEvalContext<'a> {
    fn push_issue(&mut self, issue: impl Into<BuiltinIssue>) {
        self.issues.push_issue(issue);
    }

    fn push_err(&mut self, err: impl Into<<BuiltinIssue as mucad_lang_base::Issue>::Error>) {
        self.issues.push_err(err);
    }

    fn push_warn(&mut self, warn: impl Into<<BuiltinIssue as mucad_lang_base::Issue>::Warning>) {
        self.issues.push_warn(warn);
    }

    fn push_advice(
        &mut self,
        advice: impl Into<<BuiltinIssue as mucad_lang_base::Issue>::Advice>,
    ) {
        self.issues.push_advice(advice);
    }
}

/// Built-in execution function signature
pub type BuiltinEvalFn<T = Value> =
    fn(Arguments, &mut BuiltinEvalContext) -> Result<T, BuiltinError>;

/// A type of a function returning a T as builtin.
pub type BuiltinFn<T> = fn() -> T;

/// A built-in with a static descriptor.
pub trait Builtin {
    const ITEM: &'static BuiltinItem;
}

/// A built-in which can be reconstructed from an evaluated model (primitives, operations).
pub trait BuiltinConstruct: Builtin + Sized {
    /// Returns the element descriptor of the model created by this built-in.
    fn element() -> model::Element;

    fn check_element(model: &Model) -> Result<(), BuiltinError> {
        if model.element == Self::element() {
            Ok(())
        } else {
            let default_ctx = mucad_lang_base::DefaultContext;
            Err(BuiltinError::ElementMismatch {
                expected: Self::element().to_string_with_ctx(&default_ctx),
                actual: model.element.clone().to_string_with_ctx(&default_ctx),
            })
        }
    }

    /// Constructs the Rust struct representation from a evaluated `Model`.
    fn from_model(model: &Model) -> Result<Self, BuiltinError>;
}

pub trait BuiltinPrimitiveCall: BuiltinConstruct {
    /// Call this builtin with builtin with arguments and evaluate it into a model.
    fn call(args: Arguments, _ctx: &mut BuiltinEvalContext) -> Result<Model, BuiltinError> {
        Ok(Model::new(Self::element()).with_properties(args))
    }
}

/// A built-in operation or transform, evaluating into a model tree.
pub trait BuiltinOperationCall: Builtin {
    /// Call this operation with arguments and evaluate it into a model tree.
    fn call(args: Arguments, ctx: &mut BuiltinEvalContext) -> Result<ModelTree, BuiltinError>;
}

#[macro_export]
macro_rules! construct_from_model {
    ($model:expr, $struct:ident { $( $field:ident ),* $(,)? }) => {{
        Self::check_element($model)?;
        Ok($struct {
            $(
                $field: $model.get_property_as(stringify!($field))?,
            )*
        })
    }};
}
