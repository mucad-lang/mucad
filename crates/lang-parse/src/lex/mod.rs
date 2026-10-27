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

/// The span of the literal at byte `offset` in `code`, also with `offset` right after it, e.g.
/// `10mm` in `r = 10mm;`, to change it in an editor.
///
/// A literal is a number with the unit right behind it, e.g. `10mm` or `1.5`, or a bool. A `-`
/// right in front of a number is its sign, unless it is a subtraction, e.g. not in `a-1`.
/// Numbers in strings and comments are no literals.
///
/// The literal may be invalid, e.g. `10x` with an unknown unit, which the caller checks, e.g. by
/// parsing it.
pub fn literal_at(code: &str, offset: usize) -> Option<Span> {
    let tokens: Vec<_> = lex(code).collect();
    // Whether a `-` behind the token is a subtraction.
    let is_operand = |token: &Token| {
        matches!(
            token,
            Token::Identifier(_)
                | Token::Unit(_)
                | Token::LiteralInt(_)
                | Token::LiteralFloat(_)
                | Token::LiteralBool(_)
                | Token::LiteralString(_)
                | Token::FormatStringEnd
                | Token::SigilCloseBracket
                | Token::SigilCloseSquareBracket
                | Token::SigilCloseCurlyBracket
        )
    };
    let is_whitespace_or_comment = |token: &Token| {
        matches!(
            token,
            Token::Whitespace(_)
                | Token::SingleLineComment(_)
                | Token::MultiLineComment(_)
                | Token::DocComment(_)
                | Token::InnerDocComment(_)
        )
    };

    for (index, token) in tokens.iter().enumerate() {
        let mut span = token.span.clone();
        match token.value {
            Token::LiteralBool(_) => (),
            Token::LiteralInt(_) | Token::LiteralFloat(_) => {
                // The unit right behind the number, e.g. `mm` in `10mm`.
                if let Some(unit) = tokens.get(index + 1)
                    && matches!(unit.value, Token::Unit(_) | Token::Identifier(_))
                    && unit.span.start == span.end
                {
                    span.end = unit.span.end;
                }
                // The sign right in front of the number.
                if let Some(minus) = index.checked_sub(1).map(|index| &tokens[index])
                    && minus.value == Token::OperatorSubtract
                    && minus.span.end == span.start
                {
                    let previous = tokens[..index - 1]
                        .iter()
                        .rev()
                        .find(|token| !is_whitespace_or_comment(&token.value));
                    if !previous.is_some_and(|token| is_operand(&token.value)) {
                        span.start = minus.span.start;
                    }
                }
            }
            _ => continue,
        }
        if span.start <= offset && offset <= span.end {
            return Some(span);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    use Token::{Identifier, LiteralFloat, LiteralInt, OperatorMultiply, OperatorSubtract};

    /// The tokens of `input` without whitespace.
    fn tokens_without_whitespace(input: &str) -> Vec<Token<'_>> {
        lex(input)
            .map(|token| token.value)
            .filter(|token| !matches!(token, Token::Whitespace(_)))
            .collect()
    }

    /// A `-` in front of a number is no part of it, but a subtraction or a negation, unlike the
    /// `-` of an exponent.
    #[test_case("a-1.5", vec![Identifier("a".into()), OperatorSubtract, LiteralFloat("1.5".into())] ; "subtract float")]
    #[test_case("3.0-1.5", vec![LiteralFloat("3.0".into()), OperatorSubtract, LiteralFloat("1.5".into())] ; "subtract float from float")]
    #[test_case("a-1", vec![Identifier("a".into()), OperatorSubtract, LiteralInt("1".into())] ; "subtract integer")]
    #[test_case("-1.5", vec![OperatorSubtract, LiteralFloat("1.5".into())] ; "negative float")]
    #[test_case("-.5", vec![OperatorSubtract, LiteralFloat(".5".into())] ; "negative float without leading digits")]
    #[test_case("-1e3", vec![OperatorSubtract, LiteralFloat("1e3".into())] ; "negative float with exponent")]
    #[test_case("2*-1.5", vec![LiteralInt("2".into()), OperatorMultiply, OperatorSubtract, LiteralFloat("1.5".into())] ; "multiply negative float")]
    #[test_case("1e-3", vec![LiteralFloat("1e-3".into())] ; "negative exponent")]
    #[test_case("1.5E+3", vec![LiteralFloat("1.5E+3".into())] ; "positive exponent")]
    fn minus(input: &str, expected: Vec<Token>) {
        assert_eq!(tokens_without_whitespace(input), expected);
    }

    /// The literal at the cursor `|` in `code`.
    fn literal(code: &str) -> Option<String> {
        let offset = code.find('|').expect("A cursor");
        let code = code.replacen('|', "", 1);
        literal_at(&code, offset).map(|span| code[span].to_string())
    }

    #[test_case("r = 1|0mm;", Some("10mm") ; "inside")]
    #[test_case("r = |10mm;", Some("10mm") ; "at the start")]
    #[test_case("r = 10mm|;", Some("10mm") ; "at the end")]
    #[test_case("r = 10m|m;", Some("10mm") ; "in the unit")]
    #[test_case("f(|1.5, 2)", Some("1.5") ; "argument")]
    #[test_case("r = -1|0mm;", Some("-10mm") ; "negative")]
    #[test_case("r = -1.|5mm;", Some("-1.5mm") ; "negative float")]
    #[test_case("[1, -|2]", Some("-2") ; "negative in a list")]
    #[test_case("x = a-1|0;", Some("10") ; "subtraction")]
    #[test_case("x = (a) - 1|0;", Some("10") ; "subtraction with whitespace")]
    #[test_case("x = a /* c */ -1|0;", Some("10") ; "subtraction with a comment")]
    #[test_case("x = 2*-|1;", Some("-1") ; "negative factor")]
    #[test_case("x = 1|e-3;", Some("1e-3") ; "negative exponent")]
    #[test_case("a = 45|°;", Some("45°") ; "degrees")]
    #[test_case("a = 5|0%;", Some("50%") ; "percent")]
    #[test_case("v = 1|0mm/s;", Some("10mm/s") ; "compound unit")]
    #[test_case("v = 1|0mm / 2;", Some("10mm") ; "division")]
    #[test_case("v = 1|0x;", Some("10x") ; "unknown unit")]
    #[test_case("b = tr|ue;", Some("true") ; "bool")]
    #[test_case("s = \"µ\"; r = 1|0µm;", Some("10µm") ; "after other characters")]
    #[test_case("x = x|1;", None ; "identifier")]
    #[test_case("x |= 1;", None ; "operator")]
    #[test_case("x = (|);", None ; "sigils")]
    #[test_case("s = \"1|0mm\";", None ; "in a string")]
    #[test_case("s = \"{1|0mm}\";", Some("10mm") ; "in a format expression")]
    #[test_case("r = 1mm; // 1|0mm", None ; "in a comment")]
    #[test_case("/* r = \n 1|0mm */", None ; "in a block comment")]
    fn literal_at_cursor(code: &str, expected: Option<&str>) {
        assert_eq!(literal(code).as_deref(), expected);
    }

    /// A subtraction without whitespace is parsed, also of floats.
    #[cfg(feature = "parser")]
    #[test_case("b = a-1.5;" ; "subtract float")]
    #[test_case("b = 3.0-1.5;" ; "subtract float from float")]
    #[test_case("b = a-1;" ; "subtract integer")]
    #[test_case("b = -1.5mm;" ; "negative quantity")]
    #[test_case("b = 2*-1.5;" ; "multiply negative float")]
    fn parse_minus(code: &str) {
        let source = mucad_lang_base::Source::from(code);
        assert!(crate::parse(&source).is_ok(), "{code}");
    }

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
