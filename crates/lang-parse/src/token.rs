// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use crate::Keyword;

pub use crate::lex::LexerError;

/// Source token for µcad files
#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    /// Whitespace
    Whitespace(Cow<'a, str>),

    /// A single-line comment, starting with `//`
    SingleLineComment(Cow<'a, str>),
    /// A multi-line comment, starting with `/*` and ending with `*/`
    MultiLineComment(Cow<'a, str>),
    /// A doc-comment, starting with `///`
    DocComment(Cow<'a, str>),
    /// An inner doc-comment, starting with `//!`
    InnerDocComment(Cow<'a, str>),

    /// A keyword, e.g. `fn`.
    Keyword(Keyword),

    /// An identifier, alphanumeric, starting with either an alpha character or a underscore
    Identifier(Cow<'a, str>),
    /// A unit identifier
    Unit(Cow<'a, str>),

    /// An integer literal
    LiteralInt(Cow<'a, str>),
    /// A float literal
    LiteralFloat(Cow<'a, str>),
    /// A boolean literal
    LiteralBool(bool),
    /// A string literal
    LiteralString(Cow<'a, str>),

    /// Double-quote indicating the start of a format string
    FormatStringStart,
    /// Double-quote indicating the end of a format string
    FormatStringEnd,
    /// Literal string content of a format string
    StringContent(Cow<'a, str>),
    /// Escaped character inside a format string
    Character(char),
    /// The start of the format expression inside a format string
    StringFormatOpen,
    /// The end of the format expression inside a format string
    StringFormatClose,
    /// The precision specification of the format expression inside a format string
    StringFormatPrecision(Cow<'a, str>),
    /// The width specification of the format expression inside a format string
    StringFormatWidth(Cow<'a, str>),

    /// The `:` symbol
    SigilColon,
    /// The `;` symbol
    SigilSemiColon,
    /// The `::` symbol
    SigilDoubleColon,
    /// The `(` symbol
    SigilOpenBracket,
    /// The `)` symbol
    SigilCloseBracket,
    /// The `[` symbol
    SigilOpenSquareBracket,
    /// The `]` symbol
    SigilCloseSquareBracket,
    /// The `{` symbol
    SigilOpenCurlyBracket,
    /// The `}` symbol
    SigilCloseCurlyBracket,
    /// The `#` symbol
    SigilHash,
    /// The `.` symbol
    SigilDot,
    /// The `,` symbol
    SigilComma,
    /// The `..` symbol
    SigilDoubleDot,
    /// The `@` symbol
    SigilAt,
    /// The `->` symbol
    SigilSingleArrow,
    /// The `"` symbol
    SigilQuote,

    /// Add operator: `+`
    OperatorAdd,
    /// Subtract operator: `-`
    OperatorSubtract,
    /// Multiply operator: `-`
    OperatorMultiply,
    /// Divide operator: `/`
    OperatorDivide,
    /// Union operator: `|`
    OperatorUnion,
    /// Intersect operator: `&`
    OperatorIntersect,
    /// xor operator: `^`
    OperatorPowerXor,
    /// Greater-than operator: `>`
    OperatorGreaterThan,
    /// Less-than operator: `<`
    OperatorLessThan,
    /// Greater-or-equal operator: `>=`
    OperatorGreaterEqual,
    /// Less-or-equal operator: `<=`
    OperatorLessEqual,
    /// Near operator: `~`
    OperatorNear,
    /// Equal operator: `==`
    OperatorEqual,
    /// Not-equal operator: `!=`
    OperatorNotEqual,
    /// And operator: `and`
    OperatorAnd,
    /// Or operator: `or`
    OperatorOr,
    /// Xor operator: `xor'
    OperatorXor,
    /// Not operator: `!`
    OperatorNot,
    /// Assignment operator: `=`
    OperatorAssignment,

    /// A lexer failure
    ///
    /// When encountering an error, the lexer attempts to recover and continue emitting further tokens
    Error(LexerError),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind())
    }
}

impl Token<'_> {
    /// Create an owned version of the token
    pub fn into_owned(self) -> Token<'static> {
        match self {
            Token::Whitespace(c) => Token::Whitespace(c.into_owned().into()),
            Token::SingleLineComment(c) => Token::SingleLineComment(c.into_owned().into()),
            Token::MultiLineComment(c) => Token::MultiLineComment(c.into_owned().into()),
            Token::DocComment(c) => Token::DocComment(c.into_owned().into()),
            Token::InnerDocComment(c) => Token::InnerDocComment(c.into_owned().into()),
            Token::Identifier(s) => Token::Identifier(s.into_owned().into()),
            Token::Unit(s) => Token::Unit(s.into_owned().into()),
            Token::LiteralInt(s) => Token::LiteralInt(s.into_owned().into()),
            Token::LiteralFloat(s) => Token::LiteralFloat(s.into_owned().into()),
            Token::LiteralString(s) => Token::LiteralString(s.into_owned().into()),

            Token::Keyword(keyword) => Token::Keyword(keyword),
            Token::LiteralBool(l) => Token::LiteralBool(l),
            Token::SigilColon => Token::SigilColon,
            Token::SigilSemiColon => Token::SigilSemiColon,
            Token::SigilDoubleColon => Token::SigilDoubleColon,
            Token::SigilOpenBracket => Token::SigilOpenBracket,
            Token::SigilCloseBracket => Token::SigilCloseBracket,
            Token::SigilOpenSquareBracket => Token::SigilOpenSquareBracket,
            Token::SigilCloseSquareBracket => Token::SigilCloseSquareBracket,
            Token::SigilOpenCurlyBracket => Token::SigilOpenCurlyBracket,
            Token::SigilCloseCurlyBracket => Token::SigilCloseCurlyBracket,
            Token::SigilHash => Token::SigilHash,
            Token::SigilDot => Token::SigilDot,
            Token::SigilComma => Token::SigilComma,
            Token::SigilDoubleDot => Token::SigilDoubleDot,
            Token::SigilAt => Token::SigilAt,
            Token::SigilSingleArrow => Token::SigilSingleArrow,
            Token::OperatorAdd => Token::OperatorAdd,
            Token::OperatorSubtract => Token::OperatorSubtract,
            Token::OperatorMultiply => Token::OperatorMultiply,
            Token::OperatorDivide => Token::OperatorDivide,
            Token::OperatorUnion => Token::OperatorUnion,
            Token::OperatorIntersect => Token::OperatorIntersect,
            Token::OperatorPowerXor => Token::OperatorPowerXor,
            Token::OperatorGreaterThan => Token::OperatorGreaterThan,
            Token::OperatorLessThan => Token::OperatorLessThan,
            Token::OperatorGreaterEqual => Token::OperatorGreaterEqual,
            Token::OperatorLessEqual => Token::OperatorLessEqual,
            Token::OperatorNear => Token::OperatorNear,
            Token::OperatorEqual => Token::OperatorEqual,
            Token::OperatorNotEqual => Token::OperatorNotEqual,
            Token::OperatorAnd => Token::OperatorAnd,
            Token::OperatorOr => Token::OperatorOr,
            Token::OperatorXor => Token::OperatorXor,
            Token::OperatorNot => Token::OperatorNot,
            Token::OperatorAssignment => Token::OperatorAssignment,
            Token::FormatStringStart => Token::FormatStringStart,
            Token::FormatStringEnd => Token::FormatStringEnd,
            Token::StringContent(s) => Token::StringContent(s.into_owned().into()),
            Token::Character(c) => Token::Character(c),
            Token::StringFormatOpen => Token::StringFormatOpen,
            Token::StringFormatClose => Token::StringFormatClose,
            Token::SigilQuote => Token::SigilQuote,
            Token::Error(e) => Token::Error(e),
            Token::StringFormatPrecision(c) => Token::StringFormatPrecision(c.into_owned().into()),
            Token::StringFormatWidth(c) => Token::StringFormatWidth(c.into_owned().into()),
        }
    }

    /// Get a descriptive name or symbol for the token type
    pub fn kind(&self) -> &'static str {
        match self {
            Token::Whitespace(_) => "whitespace",
            Token::SingleLineComment(_) => "single-line comment",
            Token::MultiLineComment(_) => "multi-line comment",
            Token::DocComment(_) => "doc comment",
            Token::InnerDocComment(_) => "inner doc comment",
            Token::Keyword(keyword) => keyword.as_str(),
            Token::Identifier(_) => "identifier",
            Token::Unit(_) => "unit",
            Token::LiteralInt(_) => "integer literal",
            Token::LiteralFloat(_) => "float literal",
            Token::LiteralBool(_) => "boolean literal",
            Token::LiteralString(_) => "string literal",
            Token::FormatStringStart => "start of string",
            Token::FormatStringEnd => "end of string",
            Token::StringContent(_) => "string content",
            Token::Character(_) => "escaped character",
            Token::StringFormatOpen => "string format start",
            Token::StringFormatClose => "string format end",
            Token::SigilColon => ":",
            Token::SigilSemiColon => ";",
            Token::SigilDoubleColon => "::",
            Token::SigilOpenBracket => "(",
            Token::SigilCloseBracket => ")",
            Token::SigilOpenSquareBracket => "[",
            Token::SigilCloseSquareBracket => "]",
            Token::SigilOpenCurlyBracket => "{",
            Token::SigilCloseCurlyBracket => "}",
            Token::SigilHash => "#",
            Token::SigilDot => ".",
            Token::SigilComma => ",",
            Token::SigilDoubleDot => "..",
            Token::SigilAt => "@",
            Token::SigilSingleArrow => "->",
            Token::SigilQuote => "\"",
            Token::OperatorAdd => "+",
            Token::OperatorSubtract => "-",
            Token::OperatorMultiply => "*",
            Token::OperatorDivide => "/",
            Token::OperatorUnion => "|",
            Token::OperatorIntersect => "&",
            Token::OperatorPowerXor => "^",
            Token::OperatorGreaterThan => ">",
            Token::OperatorLessThan => "<",
            Token::OperatorGreaterEqual => ">=",
            Token::OperatorLessEqual => "<=",
            Token::OperatorNear => "!",
            Token::OperatorEqual => "==",
            Token::OperatorNotEqual => "!=",
            Token::OperatorAnd => "and",
            Token::OperatorOr => "or",
            Token::OperatorXor => "xor",
            Token::OperatorNot => "not",
            Token::OperatorAssignment => "=",
            Token::StringFormatPrecision(_) => "format precision",
            Token::StringFormatWidth(_) => "format width",
            Token::Error(e) => e.kind(),
        }
    }

    /// Check if the token is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Token::Error(_))
    }
}
