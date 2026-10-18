// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{lex::from_logos::from_logos, token::Token};

use ::logos::Lexer;
use mucad_lang_base::{Span, Spanned};
use thiserror::Error;

mod from_logos;
mod logos;

/// Possible errors encountered while tokenizing
#[derive(Debug, Default, Clone, PartialEq, Error)]
pub enum LexerError {
    /// No valid token was found for the character
    #[default]
    #[error("No valid token")]
    NoValidToken,
    /// A format string was encountered that wasn't closed correctly
    #[error("Unclosed format string")]
    UnclosedStringFormat(Span),
    /// A string was encountered that wasn't closed correctly
    #[error("Unclosed string")]
    UnclosedString(Span),
}

impl LexerError {
    /// Get a descriptive name of the error type
    pub fn kind(&self) -> &'static str {
        match self {
            LexerError::NoValidToken => "no valid token",
            LexerError::UnclosedStringFormat(_) => "unclosed format string",
            LexerError::UnclosedString(_) => "unclosed string",
        }
    }
}

impl LexerError {
    /// Get the span of the error
    pub fn span(&self) -> Option<Span> {
        match self {
            LexerError::UnclosedStringFormat(span) => Some(span.clone()),
            LexerError::UnclosedString(span) => Some(span.clone()),
            _ => None,
        }
    }
}

/// Tokenize a µcad source string into an iterator of tokens.
pub fn lex<'a>(input: &'a str) -> impl Iterator<Item = Spanned<Token<'a>>> {
    from_logos(Lexer::<logos::LogosToken>::new(input).spanned())
}
