// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evaluation error

use microcad_builtin::BuiltinError;
use microcad_lang_base::{
    Identifier, Issue, LibraryId, Name, SrcRef, SrcReferrer, ToCompactString,
};
use microcad_lang_types::{Value, ValueError};
use miette::{Diagnostic, Severity};

use thiserror::Error;

use crate::ArgumentMatchError;

/// Evaluation error.
///
/// Any occuring error may not lead to stop evaluation, but will mark the evaluation as failed.
#[derive(Debug, Error, Diagnostic)]
#[allow(missing_docs)]
pub enum EvalErrorKind {
    /// An error occurred during handling values.
    #[error("Value error: {0}")]
    ValueError(#[from] ValueError),

    /// Builtin error
    #[error("Builtin error: {0}")]
    BuiltinError(#[from] BuiltinError),

    #[error("Argument match error: {err}")]
    ArgumentMatch {
        symbol_name: Name,
        context_src_ref: SrcRef,
        err: ArgumentMatchError,
    },

    /// A condition of an if statement is not a boolean
    #[error("If condition is not a boolean: {err}")]
    IfConditionIsNotBool {
        src_ref: SrcRef,

        #[label("Not a boolean")]
        condition_src_ref: SrcRef,

        err: ValueError,
    },

    /// Workbench didn't find a initialization routine matching the given arguments
    #[error("Workbench `{path}` cannot find initialization for those arguments")]
    #[diagnostic(help("Possible initializations: \n\t{}", inits.join("\n\t")))]
    NoInitializationFound {
        #[label("Got: {path}( {arguments} )")]
        src_ref: SrcRef,
        path: String,
        arguments: String,
        inits: Vec<String>,
    },

    /// Workbench didn't find a initialization routine matching the given arguments
    #[error("Workbench `{path}` has ambiguous initialization for those arguments")]
    #[diagnostic(help("Ambiguous initializations: \n\t{}", inits.join("\n\t")))]
    AmbiguousInitialization {
        #[label("Got: {path}( {arguments} )")]
        src_ref: SrcRef,
        path: String,
        arguments: String,
        inits: Vec<String>,
    },

    #[error("This expression is expected to return a value.")]
    ExpectedExpression { src_ref: SrcRef },

    #[error("This call returns a value but it is ignored")]
    CallReturnValueIgnored(SrcRef),

    #[error("Symbol `{path}` cannot be called.")]
    SymbolCannotBeCalled {
        path: String,

        #[label("Symbol name")]
        src_ref: SrcRef,
    },

    #[error("Local `{id}` will not have a value.")]
    LocalExpressionDidNotProduceAValue {
        #[label("Local `{id}` will not be set.")]
        id: Identifier,
        src_ref: SrcRef,
        expr_src_ref: SrcRef,
    },

    #[error("Local `{0}` not found in scope.")]
    LocalNotFound(Name),

    #[error("Constant expression expected")]
    ConstantExpressionExpected {
        #[label("This expression is not a constant")]
        src_ref: microcad_lang_base::SrcRef,
    },

    #[error("Unresolved symbol: {path}")]
    UnresolvedPath {
        path: String,

        #[label("This symbol could not be found in any package")]
        src_ref: SrcRef,
    },

    #[error("Built-in not found: {name}")]
    BuiltinNotFound {
        name: String,
        #[label("Built-in was called from here")]
        src_ref: SrcRef,
    },

    #[error("Built-in `{name}` is not a constant")]
    BuiltinConstantExpected {
        name: String,
        #[label("Constant expected")]
        src_ref: SrcRef,
    },

    #[error("Symbol `{symbol}` cannot be evaluated into a value")]
    SymbolNotEvaluable { symbol: String },

    #[error("Source file not loaded")]
    SourceFileNotLoaded,

    #[error("No library to evaluate")]
    NoLibraryToEvaluate,

    /// Cannot call operation without workpiece, e.g. `op()`.
    #[error("Cannot call operation `{name}` without workpiece")]
    #[diagnostic(help("Call the operation on a workpiece, e.g. `part.{name}()`"))]
    CannotCallOperationWithoutWorkpiece {
        name: String,
        #[label("Workpiece `self` is missing")]
        src_ref: SrcRef,
    },

    /// `@input` used outside of an operation.
    #[error("`@input` is only available in operations")]
    InputNotAvailable {
        #[label("Not inside an operation")]
        src_ref: SrcRef,
    },

    #[error("Library `{lib_id}` not loaded")]
    LibraryNotLoaded { lib_id: LibraryId },
}

impl SrcReferrer for EvalErrorKind {
    fn src_ref(&self) -> SrcRef {
        match self {
            EvalErrorKind::ArgumentMatch {
                context_src_ref, ..
            } => *context_src_ref,
            EvalErrorKind::IfConditionIsNotBool { src_ref, .. } => *src_ref,
            EvalErrorKind::NoInitializationFound { src_ref, .. } => *src_ref,
            EvalErrorKind::AmbiguousInitialization { src_ref, .. } => *src_ref,
            EvalErrorKind::ExpectedExpression { src_ref } => *src_ref,
            EvalErrorKind::CallReturnValueIgnored(src_ref) => *src_ref,
            EvalErrorKind::SymbolCannotBeCalled { src_ref, .. } => *src_ref,
            EvalErrorKind::LocalExpressionDidNotProduceAValue { src_ref, .. } => *src_ref,
            EvalErrorKind::ConstantExpressionExpected { src_ref } => *src_ref,
            EvalErrorKind::UnresolvedPath { src_ref, .. } => *src_ref,
            EvalErrorKind::BuiltinNotFound { src_ref, .. } => *src_ref,
            EvalErrorKind::BuiltinConstantExpected { src_ref, .. } => *src_ref,
            EvalErrorKind::CannotCallOperationWithoutWorkpiece { src_ref, .. } => *src_ref,
            EvalErrorKind::InputNotAvailable { src_ref } => *src_ref,
            _ => SrcRef::none(),
        }
    }
}

#[derive(Debug)]
pub struct EvalError(pub Box<EvalErrorKind>);

impl EvalError {
    pub fn new(err: impl Into<EvalErrorKind>) -> Self {
        Self(Box::new(err.into()))
    }

    pub fn kind(&self) -> &EvalErrorKind {
        &self.0
    }

    pub fn argument_match(
        src_ref: impl SrcReferrer,
        symbol_name: impl AsRef<str>,
        err: ArgumentMatchError,
    ) -> Self {
        Self::new(EvalErrorKind::ArgumentMatch {
            symbol_name: symbol_name.as_ref().to_compact_string(),
            context_src_ref: src_ref.src_ref(),
            err,
        })
    }
}

impl From<EvalErrorKind> for EvalError {
    fn from(kind: EvalErrorKind) -> Self {
        Self(Box::new(kind))
    }
}

impl From<BuiltinError> for EvalError {
    fn from(err: BuiltinError) -> Self {
        EvalErrorKind::from(err).into()
    }
}

impl From<ValueError> for EvalError {
    fn from(err: ValueError) -> Self {
        EvalErrorKind::from(err).into()
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for EvalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

/// Result type of any evaluation.
pub type EvalResult<T = Value> = std::result::Result<T, EvalError>;

impl From<EvalError> for miette::Report {
    fn from(value: EvalError) -> Self {
        miette::Report::new(*value.0)
    }
}

impl SrcReferrer for EvalError {
    fn src_ref(&self) -> SrcRef {
        self.0.src_ref()
    }
}

#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum EvalWarning {}

#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum EvalInfo {}

#[derive(Debug, Error, Diagnostic)]
pub enum EvalIssue {
    /// A eval error.
    #[error(transparent)]
    #[diagnostic(severity(Error), code(eval::error))]
    Err(#[from] EvalError),

    /// A warning from the eval.
    #[error(transparent)]
    #[diagnostic(severity(Warning), code(eval::warning))]
    Warn(#[from] EvalWarning),

    /// An info from the eval.
    #[error(transparent)]
    #[diagnostic(severity(Advice), code(eval::info))]
    Info(#[from] EvalInfo),
}

impl Issue for EvalIssue {
    type Err = EvalError;
    type Warn = EvalWarning;
    type Info = EvalInfo;

    fn severity(&self) -> Severity {
        match self {
            EvalIssue::Err(_) => Severity::Error,
            EvalIssue::Warn(_) => Severity::Warning,
            EvalIssue::Info(_) => Severity::Advice,
        }
    }
}
