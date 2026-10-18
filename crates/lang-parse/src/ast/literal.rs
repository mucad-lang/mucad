// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ast::{ItemExtras, Span, Unit};

use mucad_lang_base::CompactString;
use mucad_macros::Visit;
use serde::{Serialize, Serializer};
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;

/// A literal value
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub struct Literal {
    pub span: Span,
    pub extras: ItemExtras,
    pub literal: LiteralKind,
}

/// The various types of literal values a [`Literal`] can contain
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
pub enum LiteralKind {
    #[visit(skip)]
    Error(LiteralError),
    String(StringLiteral),
    Bool(BoolLiteral),
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    Quantity(QuantityLiteral),
}

impl LiteralKind {
    /// Get the span for the literal
    pub fn span(&self) -> Span {
        match self {
            LiteralKind::Error(lit) => lit.span.clone(),
            LiteralKind::String(lit) => lit.span.clone(),
            LiteralKind::Bool(lit) => lit.span.clone(),
            LiteralKind::Integer(lit) => lit.span.clone(),
            LiteralKind::Float(lit) => lit.span.clone(),
            LiteralKind::Quantity(lit) => lit.span.clone(),
        }
    }
}

/// A string literal, without format expressions
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct StringLiteral {
    pub span: Span,
    pub content: String,
}

/// A boolean literal, either `true` or `false`
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct BoolLiteral {
    pub span: Span,
    pub value: bool,
}

/// An integer literal without type
#[derive(Debug, Hash, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct IntegerLiteral {
    pub span: Span,
    pub value: CompactString,
}

/// An float literal without type
#[derive(Debug, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct FloatLiteral {
    pub span: Span,
    pub value: CompactString,
}

impl std::hash::Hash for FloatLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

/// A float literal with type
#[derive(Debug, PartialEq, Visit, Serialize)]
#[allow(missing_docs)]
#[visit(default)]
pub struct QuantityLiteral {
    pub span: Span,
    pub value: CompactString,
    pub unit: Unit,
}

impl std::hash::Hash for QuantityLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.unit.hash(state);
    }
}

/// An error that can be encountered while parsing literal tokens
#[derive(Debug, Hash, PartialEq, Clone, Serialize)]
#[allow(missing_docs)]
pub struct LiteralError {
    pub span: Span,
    pub kind: LiteralErrorKind,
}

#[derive(Debug, Error, PartialEq, Clone)]
#[allow(missing_docs)]
pub enum LiteralErrorKind {
    #[error(transparent)]
    Float(#[from] ParseFloatError),
    #[error(transparent)]
    Int(#[from] ParseIntError),
    #[error("unclosed string literal")]
    UnclosedString,
    #[error("only numeric literals can be typed")]
    Untypable,
}

impl std::hash::Hash for LiteralErrorKind {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        // TODO implement
    }
}

impl Serialize for LiteralErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
