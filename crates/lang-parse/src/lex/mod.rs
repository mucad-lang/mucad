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

/// Options of [`lex_with`].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LexOptions {
    /// Remove comments, also doc comments. A comment is replaced by a space, so the tokens
    /// before and after it stay separated.
    pub strip_comments: bool,
    /// Replace whitespace with the whitespace of formatted code: a space, a new line or an
    /// empty line, see [`normalized_whitespace`].
    pub normalize_whitespace: bool,
}

/// The whitespace of formatted code for whitespace: `" "` without a new line, `"\n"` for one
/// new line and an empty line `"\n\n"` for more new lines.
pub fn normalized_whitespace(whitespace: &str) -> &'static str {
    match whitespace.matches('\n').count() {
        0 => " ",
        1 => "\n",
        _ => "\n\n",
    }
}

/// Tokenize a µcad source string with `options`, e.g. without comments.
///
/// Consecutive whitespace, also of removed comments, is one whitespace token.
pub fn lex_with<'a>(
    input: &'a str,
    options: LexOptions,
) -> impl Iterator<Item = Spanned<Token<'a>>> {
    let is_comment = |token: &Token| {
        matches!(
            token,
            Token::SingleLineComment(_)
                | Token::MultiLineComment(_)
                | Token::DocComment(_)
                | Token::InnerDocComment(_)
        )
    };
    // The text of whitespace or of a removed comment.
    let whitespace = move |token: &Token<'a>| match token {
        Token::Whitespace(text) => Some(text.clone()),
        token if options.strip_comments && is_comment(token) => Some(" ".into()),
        _ => None,
    };

    let mut tokens = lex(input).peekable();
    std::iter::from_fn(move || {
        let token = tokens.next()?;
        let Some(mut text) = whitespace(&token) else {
            return Some(token);
        };
        let mut span = token.span;
        while let Some(next) = tokens.peek()
            && let Some(next_text) = whitespace(next)
        {
            text.to_mut().push_str(&next_text);
            span.end = next.span.end;
            tokens.next();
        }
        if options.normalize_whitespace {
            text = normalized_whitespace(&text).into();
        }
        Some(Spanned::new(span, Token::Whitespace(text)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tokens of `input` lexed with `options`.
    fn tokens(input: &str, options: LexOptions) -> Vec<Token<'_>> {
        lex_with(input, options).map(|token| token.value).collect()
    }

    /// Comments are removed and whitespace is normalized.
    #[test]
    fn lex_with_options() {
        let input = "a = 1; // one\n\n\n  /* two */b/*three*/c";
        let none = LexOptions::default();
        assert_eq!(
            tokens(input, none),
            lex(input).map(|token| token.value).collect::<Vec<_>>()
        );

        let strip = LexOptions {
            strip_comments: true,
            normalize_whitespace: false,
        };
        let stripped = tokens(input, strip);
        assert_eq!(stripped[6], Token::Whitespace("  \n\n\n   ".into()));
        assert_eq!(stripped[8], Token::Whitespace(" ".into()));
        assert!(
            !stripped
                .iter()
                .any(|token| matches!(token, Token::SingleLineComment(_)))
        );

        let normalize = LexOptions {
            strip_comments: true,
            normalize_whitespace: true,
        };
        let normalized = tokens(input, normalize);
        assert_eq!(normalized[6], Token::Whitespace("\n\n".into()));
        assert_eq!(
            &normalized[7..],
            [
                Token::Identifier("b".into()),
                Token::Whitespace(" ".into()),
                Token::Identifier("c".into()),
            ]
        );
        // The span of merged whitespace covers the removed comments.
        let spans: Vec<_> = lex_with(input, normalize).map(|token| token.span).collect();
        assert_eq!(spans[6], 6..27);
    }
}
