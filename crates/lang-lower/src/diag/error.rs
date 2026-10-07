// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_builtin::BuiltinError;
use microcad_lang_base::{Identifier, IdentifierList, Refer, SrcRef, SrcReferrer};
use microcad_lang_parse::ast;
use microcad_lang_types::{TypeError, ValueError};
use miette::Diagnostic;
use thiserror::Error;

use crate::ir;

/// Errors and warnings during lowering
#[derive(Debug, Clone, Error, Diagnostic)]
#[allow(missing_docs)]
pub enum LowerError {
    #[error("Error parsing integer literal: {0}")]
    ParseIntError(#[label("{0}")] Refer<std::num::ParseIntError>),

    #[error("Value error: {0}")]
    ValueError(#[from] ValueError),

    #[error("Built-in error: {0}")]
    BuiltinError(#[from] BuiltinError),

    #[error("Unknown unit: {0}")]
    UnknownUnit(#[label("Unknown unit")] Refer<String>),

    #[error("Duplicate argument: {id}")]
    DuplicateArgument {
        #[label(primary, "Duplicate argument")]
        id: Identifier,
        #[label("Previous declaration")]
        previous: Identifier,
    },

    #[error("Inner doc comments must appear before inner attributes")]
    #[diagnostic(
        code(lower::inner_doc_after_inner_attr),
        help("Move this doc comment to the top of the block")
    )]
    InnerDocAfterInnerAttribute {
        #[label("this doc comment is out of order")]
        src_ref: SrcRef,
    },

    #[error("Inner attributes must appear before statements")]
    #[diagnostic(
        code(lower::inner_attr_after_stmt),
        help("Move this attribute above the first statement")
    )]
    InnerAttributeAfterStatement {
        #[label("this attribute is out of order")]
        src_ref: SrcRef,
    },
    /// Statements that are truly forbidden in a block
    #[error("This statement is not allowed here")]
    #[diagnostic(code(lower::unexpected_statement))]
    StatementNotAllowed {
        #[label("unexpected statement")]
        src_ref: SrcRef,
    },

    /// Grammar rule error
    #[error("Invalid id '{0}'")]
    InvalidIdentifier(Refer<String>),

    #[error("Unknown type: {0}")]
    UnknownType(#[label("Unknown type")] Refer<String>),

    /// A Type error
    #[error("Type error: {0}")]
    TypeError(#[from] Refer<TypeError>),

    /// Invalid glob pattern
    #[error("Invalid glob pattern, wildcard must be at the end of the pattern")]
    InvalidGlobPattern(SrcRef),

    /// A glob import is given an alias
    #[error("Glob imports can't be given an alias")]
    UseGlobAlias(SrcRef),

    /// An invalid literal was encountered
    #[error("Invalid literal: {error}")]
    InvalidLiteral {
        error: ast::LiteralErrorKind,
        #[label("{error}")]
        src_ref: SrcRef,
    },

    /// An invalid expression was encountered
    #[error("Invalid expression")]
    InvalidExpression { src_ref: SrcRef },

    /// A type range between non-integer literals
    #[error("range expressions must be between integers")]
    InvalidRangeType { src_ref: SrcRef },

    /// Implicit returns in tail expressions are treated as regular statements inside workbenches
    #[error("Ignored implicit return in workbench")]
    #[diagnostic(help("Add a trailing semicolon to remove the implicit return"))]
    ImplicitWorkbenchReturn {
        #[label("Workbenches don't return any value")]
        src_ref: SrcRef,
    },

    #[error("Statement is unreachable")]
    #[diagnostic(help("Remove this statement {src_ref}"), severity = "Warning")]
    Unreachable {
        #[label("Last statement to be evaluated")]
        last_ref: SrcRef,
        #[label("Statement")]
        src_ref: SrcRef,
    },

    #[error("This is not a constant expression")]
    InvalidConstantExpression {
        #[label("Expression")]
        src_ref: SrcRef,
    },

    #[error("The result of the function statement is ignored")]
    FunctionStatementIgnored(#[label("Removed this statement")] SrcRef),

    #[error("Unsupported command attribute: {path}")]
    UnsupportedCommandAttribute {
        path: ir::Path,
        #[label("Command attribute")]
        src_ref: SrcRef,
    },

    #[error("Unsupported key-value attribute: {key}")]
    UnsupportedKeyValueAttribute {
        key: ir::Path,
        #[label("Command attribute")]
        src_ref: SrcRef,
    },

    #[error("Unsupported tag attribute: {tag}")]
    UnsupportedTagAttribute {
        #[label("Tag attribute")]
        tag: ir::Identifier,
    },

    /// Type annotation not allowed (in an if statement)
    #[error("Invalid initializer")]
    InvalidInitStatement {
        #[label("Invalid initializer statement")]
        stmt_src_ref: SrcRef,

        #[label("Remove this type annotation")]
        src_ref: SrcRef,
    },

    #[error("This is not a workbench input property: {name}")]
    #[diagnostic(code(lower::not_an_input_property))]
    NotAnInputProperty {
        name: microcad_lang_base::Identifier,
        #[help("Possible inputs")]
        possible_inputs: IdentifierList,
    },

    #[error("Input `{name}` not initialized")]
    #[diagnostic(code(lower::input_not_initalized))]
    InputNotInitialized {
        name: Identifier,
        src_ref: SrcRef,
        #[label("The input to be initialized has been defined here")]
        param_src_ref: SrcRef,
    },

    #[error("This initializer is equivant to the default initializer and will not be called")]
    #[diagnostic(code(lower::duplicated_default_initializer), severity = "Warning")]
    DuplicatedDefaultInitializer { src_ref: SrcRef },

    #[error("Conflicting stability attributes found")]
    #[diagnostic(
        code(microcad::lower::conflicting_stability),
        help(
            "A symbol can only have one stability level (e.g., choose between #[stable], #[experimental], or #[deprecated])"
        )
    )]
    ConflictingStability {
        #[label("conflicting attribute here")]
        src_ref: SrcRef,
    },

    #[error("Duplicate #[{attr}] attribute")]
    #[diagnostic(code(microcad::lower::duplicate_attribute))]
    DuplicateAttribute {
        attr: String,
        #[label("duplicate attribute")]
        src_ref: SrcRef,
    },

    #[error("Invalid version format '{value}'")]
    #[diagnostic(
        code(microcad::lower::invalid_version_string),
        help("Version must follow semantic versioning, e.g., \"0.2.0\"")
    )]
    InvalidVersionString {
        value: String,
        #[label("invalid version string")]
        src_ref: SrcRef,
    },

    #[error("Attribute #[{attr}] requires a '{arg}' argument")]
    #[diagnostic(
        code(microcad::lower::missing_attribute_argument),
        help("Provide the argument like #[{attr}({arg} = \"...\")]")
    )]
    MissingAttributeArgument {
        attr: String,
        arg: String,
        #[label("missing argument")]
        src_ref: SrcRef,
    },
}

impl SrcReferrer for LowerError {
    fn src_ref(&self) -> SrcRef {
        use LowerError::*;
        match self {
            ValueError(_) | BuiltinError(_) => SrcRef::none(),
            DuplicateArgument { id, .. } => id.src_ref(),
            StatementNotAllowed { src_ref }
            | InvalidGlobPattern(src_ref)
            | UseGlobAlias(src_ref)
            | InvalidLiteral { src_ref, .. }
            | InvalidExpression { src_ref }
            | InvalidRangeType { src_ref }
            | ImplicitWorkbenchReturn { src_ref } => *src_ref,
            ParseIntError(parse_int_error) => parse_int_error.src_ref(),
            InvalidIdentifier(id) => id.src_ref(),
            UnknownUnit(unit) => unit.src_ref(),
            UnknownType(ty) => ty.src_ref(),
            TypeError(ty) => ty.src_ref(),
            Unreachable { src_ref, .. } => *src_ref,
            InvalidConstantExpression { src_ref } => *src_ref,
            InnerDocAfterInnerAttribute { src_ref } => *src_ref,
            InnerAttributeAfterStatement { src_ref } => *src_ref,
            FunctionStatementIgnored(src_ref) => *src_ref,
            UnsupportedCommandAttribute { src_ref, .. } => *src_ref,
            UnsupportedKeyValueAttribute { src_ref, .. } => *src_ref,
            UnsupportedTagAttribute { tag } => tag.src_ref(),
            InvalidInitStatement { src_ref, .. } => *src_ref,
            InputNotInitialized { src_ref, .. } => *src_ref,
            NotAnInputProperty { name, .. } => name.src_ref(),
            DuplicatedDefaultInitializer { src_ref } => *src_ref,
            ConflictingStability { src_ref } => *src_ref,
            DuplicateAttribute { src_ref, .. } => *src_ref,
            InvalidVersionString { src_ref, .. } => *src_ref,
            MissingAttributeArgument { src_ref, .. } => *src_ref,
        }
    }
}
