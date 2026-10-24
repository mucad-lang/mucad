// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::lex::LexerError;

use logos::{Lexer, Logos, internal::LexerInternal};
use mucad_lang_base::{Span, Spanned};
use std::borrow::Cow;

#[derive(Logos, Debug, PartialEq, Clone)]
#[allow(missing_docs)]
#[logos(error(LexerError))]
pub enum LogosToken<'a> {
    #[regex(r"[ \t\r\n\f]", callback = whitespace_callback)]
    Whitespace(Cow<'a, str>),

    #[regex(r#"\/\/\/[^\n]*"#, allow_greedy = true, callback = token_cow)]
    DocComment(Cow<'a, str>),
    #[regex(r#"\/\/![^\n]*"#, allow_greedy = true, callback = token_cow)]
    InnerDocComment(Cow<'a, str>),
    #[regex(r#"\/\/[^\n]*"#, allow_greedy = true, callback = single_line_comment_callback)]
    SingleLineComment(Cow<'a, str>),
    #[token("/*", callback = multi_line_comment_callback)]
    MultiLineComment(Cow<'a, str>),

    #[regex("(_|[a-zA-Z])[_a-zA-Z0-9']*", callback = token_cow, priority = 4)]
    Identifier(Cow<'a, str>),
    // `µ` (U+00B5) is the prefix of micro units, e.g. `µm` or `µl`.
    #[regex(r#"(µ?[a-z]+[²³23]?(/[a-z]+[²³23]?)?)|°|%|'"#, callback = token_cow, priority = 3)]
    Unit(Cow<'a, str>),

    #[regex(r#"(0|[1-9]\d*)"#, callback = token_cow)]
    LiteralInt(Cow<'a, str>),
    // to distinguish if the dot after a number is part of a float, or the start of a range operator
    #[regex(r#"(0|[1-9]\d*)\.\."#, callback = token_cow)]
    LiteralIntWithRange(Cow<'a, str>),
    // ~3 times the same regex, with either the leading digits, trailing digits or exponent required
    // this ensures we have all parts optional, but don't match a lone dot
    #[regex(r#"((0|[1-9]\d*)\.(\d+)?((e|E)(-|\+)?(\d+))?)|(-?(0|[1-9]\d*)?\.(\d+)((e|E)(-|\+)?(\d+))?)|(-?(0|[1-9]\d*)\.?((e|E)(-|\+)?(\d+)))"#, callback = token_cow)]
    LiteralFloat(Cow<'a, str>),
    #[regex(r#"""#, string_token_callback)]
    Quote(QuoteVariant<'a>),
    #[token("true")]
    LiteralBoolTrue,
    #[token("false")]
    LiteralBoolFalse,

    #[token(":")]
    SigilColon,
    #[token(";")]
    SigilSemiColon,
    #[token("::")]
    SigilDoubleColon,
    #[token("(")]
    SigilOpenBracket,
    #[token(")")]
    SigilCloseBracket,
    #[token("[")]
    SigilOpenSquareBracket,
    #[token("]")]
    SigilCloseSquareBracket,
    #[token("{")]
    SigilOpenCurlyBracket,
    #[token("}")]
    SigilCloseCurlyBracket,
    #[token("#")]
    SigilHash,
    #[token(".")]
    SigilDot,
    #[token(",")]
    SigilComma,
    #[token("..")]
    SigilDoubleDot,
    #[token("@")]
    SigilAt,
    #[token("->")]
    SigilSingleArrow,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSubtract,
    #[token("*")]
    OperatorMultiply,
    #[token("/")]
    OperatorDivide,
    #[token("|")]
    OperatorUnion,
    #[token("&")]
    OperatorIntersect,
    #[token("^")]
    OperatorPowerXor,
    #[token(">")]
    OperatorGreaterThan,
    #[token("<")]
    OperatorLessThan,
    #[token(">=")]
    OperatorGreaterEqual,
    #[token("<=")]
    OperatorLessEqual,
    #[token("~")]
    OperatorNear,
    #[token("==")]
    OperatorEqual,
    #[token("!=")]
    OperatorNotEqual,
    #[token("and")]
    OperatorAnd,
    #[token("or", priority = 5)]
    OperatorOr,
    #[token("xor")]
    OperatorXor,
    #[token("!")]
    OperatorNot,
    #[token("=")]
    OperatorAssignment,
    Error(LexerError),
}

#[derive(Debug, PartialEq, Clone)]
pub enum QuoteVariant<'a> {
    String {
        span: Span,
        contents: Vec<Spanned<StringToken<'a>>>,
    },
    Unit,
}

fn multi_line_comment_callback<'a>(lex: &mut Lexer<'a, LogosToken<'a>>) -> Option<Cow<'a, str>> {
    let text = &lex.source()[lex.span().start..];
    let end = text.find("*/")?;
    let comment = text[2..end].trim_start_matches('*').trim();
    lex.bump(end);
    Some(comment.into())
}

fn whitespace_callback<'a>(lex: &mut Lexer<'a, LogosToken<'a>>) -> Option<Cow<'a, str>> {
    let text = &lex.source()[lex.span().start..];
    let end = text
        .find(|c: char| !c.is_ascii_whitespace())
        .unwrap_or(text.len());
    let whitespace = &text[0..end];
    lex.bump(end - 1);
    Some(whitespace.into())
}

fn single_line_comment_callback<'a>(lex: &mut Lexer<'a, LogosToken<'a>>) -> Option<Cow<'a, str>> {
    Some(lex.slice().trim().into())
}

fn string_token_callback<'a>(
    lex: &mut Lexer<'a, LogosToken<'a>>,
) -> Result<QuoteVariant<'a>, LexerError> {
    // if we have a quote that follow then end of a number (digit or '.') or list, the token is an inch unit
    // this is a massive hack, but the best I can think of to distinguish '"
    let last_byte = lex
        .source()
        .as_bytes()
        .get(lex.span().start.saturating_sub(1))
        .copied()
        .unwrap_or_default();
    if last_byte == b']' || last_byte == b'.' || last_byte.is_ascii_digit() {
        return Ok(QuoteVariant::Unit);
    }

    let span_start = lex.span().start;
    let mut string_lexer = lex.clone().morph::<StringToken>();
    let mut tokens = Vec::new();
    while let Some(token) = string_lexer.next() {
        match token {
            Ok(StringToken::Quote) => {
                *lex = string_lexer.morph();
                let span_end = lex.span().end;
                return Ok(QuoteVariant::String {
                    span: span_start..span_end,
                    contents: tokens,
                });
            }
            Err(e) => {
                let start = lex.span().start;
                *lex = string_lexer.morph();
                return Err(match e {
                    LexerError::UnclosedStringFormat(span) => {
                        LexerError::UnclosedStringFormat(start..span.end)
                    }
                    e => e,
                });
            }
            Ok(value) => tokens.push(Spanned {
                span: string_lexer.span(),
                value,
            }),
        }
    }

    Err(LexerError::UnclosedString(lex.span()))
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(LexerError))]
#[allow(missing_docs)]
pub enum StringToken<'a> {
    #[regex(r#"[^"{}\\]+"#, callback = token_cow)]
    Content(Cow<'a, str>),
    #[regex(r#"\\["\\/bfnrt]"#, callback = escaped_car)]
    Escaped(char),
    #[token(r#"\"#)]
    BackSlash,
    #[token(r#"{{"#)]
    EscapedCurlyOpen,
    #[token(r#"}}"#)]
    EscapedCurlyClose,
    #[token("{", format_token_callback)]
    FormatStart(
        (
            Vec<Spanned<LogosToken<'a>>>,
            Vec<Spanned<StringFormatToken<'a>>>,
        ),
    ),
    #[token(r#"""#)]
    Quote,
}

/// Get the literal value of string tokens, if the string is a literal
pub fn get_literal_string(string_tokens: &[Spanned<StringToken>]) -> Option<String> {
    let mut result = String::new();
    for token in string_tokens {
        match &token.value {
            StringToken::Content(s) => result.push_str(s.as_ref()),
            StringToken::Escaped(c) => result.push(*c),
            StringToken::BackSlash => result.push('\\'),
            StringToken::EscapedCurlyOpen => result.push('{'),
            StringToken::EscapedCurlyClose => result.push('}'),
            _ => return None,
        }
    }

    Some(result)
}

#[allow(clippy::type_complexity)]
fn format_token_callback<'a>(
    lex: &mut Lexer<'a, StringToken<'a>>,
) -> Result<
    (
        Vec<Spanned<LogosToken<'a>>>,
        Vec<Spanned<StringFormatToken<'a>>>,
    ),
    LexerError,
> {
    let mut expression_lexer = lex.clone().morph::<LogosToken>();
    let mut expression_tokens = Vec::new();
    let mut format_tokens = Vec::new();

    let mut with_format = false;
    while let Some(token) = expression_lexer.next() {
        match token {
            Ok(LogosToken::SigilCloseCurlyBracket) => break,
            Ok(LogosToken::SigilColon) => {
                with_format = true;
                break;
            }
            Ok(LogosToken::Quote(QuoteVariant::String { contents, .. })) => {
                let start = lex.span().start;
                let end = contents
                    .first()
                    .map(|t| t.span.start)
                    .unwrap_or_else(|| expression_lexer.span().start);
                lex.end(end);
                return Err(LexerError::UnclosedStringFormat(start..end));
            }
            Err(LexerError::UnclosedString(span)) => {
                let start = lex.span().start;
                let end = span.start + 1;
                lex.end(end);
                return Err(LexerError::UnclosedStringFormat(start..end));
            }
            Err(e) => return Err(e),
            Ok(value) => expression_tokens.push(Spanned {
                span: expression_lexer.span(),
                value,
            }),
        }
    }

    let mut format_lexer = expression_lexer.morph::<StringFormatToken>();
    if with_format {
        while let Some(token) = format_lexer.next() {
            match token {
                Ok(StringFormatToken::FormatEnd) => {
                    break;
                }
                Err(e) => {
                    *lex = format_lexer.morph();
                    return Err(e);
                }
                Ok(StringFormatToken::StringEnd) => {
                    let start = lex.span().start;
                    let end = format_lexer.span().end;
                    lex.end(end);
                    return Err(LexerError::UnclosedStringFormat(start..end));
                }
                Ok(value) => format_tokens.push(Spanned {
                    span: format_lexer.span(),
                    value,
                }),
            }
        }
    }
    *lex = format_lexer.morph();

    Ok((expression_tokens, format_tokens))
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(LexerError))]
#[allow(missing_docs)]
pub enum StringFormatToken<'a> {
    #[token("}")]
    FormatEnd,
    #[regex(r#"\.[\d]+"#, callback = token_cow)]
    FormatPrecision(Cow<'a, str>),
    #[regex(r#"0[\d]+"#, callback = token_cow)]
    FormatWidth(Cow<'a, str>),
    #[token("\"")]
    StringEnd,
}

fn token_cow<'a, Token: Logos<'a, Source = str>>(lex: &mut Lexer<'a, Token>) -> Cow<'a, str> {
    Cow::Borrowed(lex.slice())
}

fn escaped_car<'a, Token: Logos<'a, Source = str>>(lex: &mut Lexer<'a, Token>) -> char {
    lex.slice()
        .chars()
        .nth(1)
        .expect("escaped token always has 2 chars")
}
