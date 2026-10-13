// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::lex::logos::{
    LogosToken, QuoteVariant, StringFormatToken, StringToken, get_literal_string,
};
use crate::token::Token;
use either::Either;
use logos::SpannedIter;
use microcad_lang_base::Spanned;
use std::borrow::Cow;
use std::iter::once;

/// map the lexed tokens from logos into the output tokens, allowing for some post-processing to help the ast building
pub(crate) fn from_logos<'source>(
    tokens: SpannedIter<'source, LogosToken<'source>>,
) -> impl Iterator<Item = Spanned<Token<'source>>> {
    tokens
        .map(|(token, span)| match token {
            Ok(token) => Spanned::new(span, token),
            Err(error) => Spanned::new(error.span().unwrap_or(span), LogosToken::Error(error)),
        })
        .flat_map(map_token)
}

fn map_token(token: Spanned<LogosToken>) -> impl Iterator<Item = Spanned<Token>> {
    match token.value {
        LogosToken::Whitespace(c) => {
            Either::Left(once(Spanned::new(token.span, Token::Whitespace(c))))
        }
        LogosToken::SingleLineComment(c) => {
            Either::Left(once(Spanned::new(token.span, Token::SingleLineComment(c))))
        }
        LogosToken::MultiLineComment(c) => {
            Either::Left(once(Spanned::new(token.span, Token::MultiLineComment(c))))
        }
        LogosToken::DocComment(c) => {
            Either::Left(once(Spanned::new(token.span, Token::DocComment(c))))
        }
        LogosToken::InnerDocComment(c) => {
            Either::Left(once(Spanned::new(token.span, Token::InnerDocComment(c))))
        }
        LogosToken::KeywordMod => Either::Left(once(Spanned::new(token.span, Token::KeywordMod))),
        LogosToken::KeywordPart => Either::Left(once(Spanned::new(token.span, Token::KeywordPart))),
        LogosToken::KeywordSketch => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordSketch)))
        }
        LogosToken::KeywordOp => Either::Left(once(Spanned::new(token.span, Token::KeywordOp))),
        LogosToken::KeywordFn => Either::Left(once(Spanned::new(token.span, Token::KeywordFn))),
        LogosToken::KeywordIf => Either::Left(once(Spanned::new(token.span, Token::KeywordIf))),
        LogosToken::KeywordElse => Either::Left(once(Spanned::new(token.span, Token::KeywordElse))),
        LogosToken::KeywordUse => Either::Left(once(Spanned::new(token.span, Token::KeywordUse))),
        LogosToken::KeywordAs => Either::Left(once(Spanned::new(token.span, Token::KeywordAs))),
        LogosToken::KeywordReturn => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordReturn)))
        }
        LogosToken::KeywordPub => Either::Left(once(Spanned::new(token.span, Token::KeywordPub))),
        LogosToken::KeywordConst => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordConst)))
        }
        LogosToken::KeywordProp => Either::Left(once(Spanned::new(token.span, Token::KeywordProp))),
        LogosToken::KeywordInit => Either::Left(once(Spanned::new(token.span, Token::KeywordInit))),
        LogosToken::KeywordSuper => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordSuper)))
        }
        LogosToken::KeywordPlugin => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordPlugin)))
        }
        LogosToken::KeywordAssembly => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordAssembly)))
        }
        LogosToken::KeywordMaterial => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordMaterial)))
        }
        LogosToken::KeywordUnit => Either::Left(once(Spanned::new(token.span, Token::KeywordUnit))),
        LogosToken::KeywordEnum => Either::Left(once(Spanned::new(token.span, Token::KeywordEnum))),
        LogosToken::KeywordStruct => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordStruct)))
        }
        LogosToken::KeywordMatch => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordMatch)))
        }
        LogosToken::KeywordType => Either::Left(once(Spanned::new(token.span, Token::KeywordType))),
        LogosToken::KeywordExtern => {
            Either::Left(once(Spanned::new(token.span, Token::KeywordExtern)))
        }
        LogosToken::Identifier(i) => {
            Either::Left(once(Spanned::new(token.span, Token::Identifier(i))))
        }
        LogosToken::Unit(u) => Either::Left(once(Spanned::new(token.span, Token::Unit(u)))),
        LogosToken::LiteralInt(l) => {
            Either::Left(once(Spanned::new(token.span, Token::LiteralInt(l))))
        }
        LogosToken::LiteralIntWithRange(l) => Either::Right(Box::new(
            once(Spanned::new(
                token.span.start..(token.span.end - 2),
                Token::LiteralInt(match l {
                    Cow::Borrowed(l) => Cow::Borrowed(&l[0..l.len() - 2]),
                    Cow::Owned(mut l) => {
                        l.truncate(l.len() - 2);
                        Cow::Owned(l)
                    }
                }),
            ))
            .chain(once(Spanned::new(
                (token.span.end - 2)..token.span.end,
                Token::SigilDoubleDot,
            ))),
        )
            as Box<dyn Iterator<Item = Spanned<Token>>>),
        LogosToken::LiteralFloat(l) => {
            Either::Left(once(Spanned::new(token.span, Token::LiteralFloat(l))))
        }
        LogosToken::Quote(QuoteVariant::Unit) => {
            Either::Left(once(Spanned::new(token.span, Token::SigilQuote)))
        }
        LogosToken::Quote(QuoteVariant::String { span, contents }) => {
            Either::Right(match get_literal_string(&contents) {
                Some(literal) => Box::new(once(Spanned::new(
                    span,
                    Token::LiteralString(literal.into()),
                ))) as Box<dyn Iterator<Item = Spanned<Token>>>,
                None => Box::new(
                    once(Spanned::new(
                        span.start..(span.start + 1),
                        Token::FormatStringStart,
                    ))
                    .chain(contents.into_iter().flat_map(map_string_token))
                    .chain(once(Spanned::new(
                        (span.end - 1)..span.end,
                        Token::FormatStringEnd,
                    ))),
                ) as Box<dyn Iterator<Item = Spanned<Token>>>,
            })
        }
        LogosToken::LiteralBoolTrue => {
            Either::Left(once(Spanned::new(token.span, Token::LiteralBool(true))))
        }
        LogosToken::LiteralBoolFalse => {
            Either::Left(once(Spanned::new(token.span, Token::LiteralBool(false))))
        }
        LogosToken::SigilColon => Either::Left(once(Spanned::new(token.span, Token::SigilColon))),
        LogosToken::SigilSemiColon => {
            Either::Left(once(Spanned::new(token.span, Token::SigilSemiColon)))
        }
        LogosToken::SigilDoubleColon => {
            Either::Left(once(Spanned::new(token.span, Token::SigilDoubleColon)))
        }
        LogosToken::SigilOpenBracket => {
            Either::Left(once(Spanned::new(token.span, Token::SigilOpenBracket)))
        }
        LogosToken::SigilCloseBracket => {
            Either::Left(once(Spanned::new(token.span, Token::SigilCloseBracket)))
        }
        LogosToken::SigilOpenSquareBracket => Either::Left(once(Spanned::new(
            token.span,
            Token::SigilOpenSquareBracket,
        ))),
        LogosToken::SigilCloseSquareBracket => Either::Left(once(Spanned::new(
            token.span,
            Token::SigilCloseSquareBracket,
        ))),
        LogosToken::SigilOpenCurlyBracket => {
            Either::Left(once(Spanned::new(token.span, Token::SigilOpenCurlyBracket)))
        }
        LogosToken::SigilCloseCurlyBracket => Either::Left(once(Spanned::new(
            token.span,
            Token::SigilCloseCurlyBracket,
        ))),
        LogosToken::SigilHash => Either::Left(once(Spanned::new(token.span, Token::SigilHash))),
        LogosToken::SigilDot => Either::Left(once(Spanned::new(token.span, Token::SigilDot))),
        LogosToken::SigilComma => Either::Left(once(Spanned::new(token.span, Token::SigilComma))),
        LogosToken::SigilDoubleDot => {
            Either::Left(once(Spanned::new(token.span, Token::SigilDoubleDot)))
        }
        LogosToken::SigilAt => Either::Left(once(Spanned::new(token.span, Token::SigilAt))),
        LogosToken::SigilSingleArrow => {
            Either::Left(once(Spanned::new(token.span, Token::SigilSingleArrow)))
        }
        LogosToken::OperatorAdd => Either::Left(once(Spanned::new(token.span, Token::OperatorAdd))),
        LogosToken::OperatorSubtract => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorSubtract)))
        }
        LogosToken::OperatorMultiply => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorMultiply)))
        }
        LogosToken::OperatorDivide => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorDivide)))
        }
        LogosToken::OperatorUnion => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorUnion)))
        }
        LogosToken::OperatorIntersect => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorIntersect)))
        }
        LogosToken::OperatorPowerXor => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorPowerXor)))
        }
        LogosToken::OperatorGreaterThan => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorGreaterThan)))
        }
        LogosToken::OperatorLessThan => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorLessThan)))
        }
        LogosToken::OperatorGreaterEqual => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorGreaterEqual)))
        }
        LogosToken::OperatorLessEqual => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorLessEqual)))
        }
        LogosToken::OperatorNear => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorNear)))
        }
        LogosToken::OperatorEqual => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorEqual)))
        }
        LogosToken::OperatorNotEqual => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorNotEqual)))
        }
        LogosToken::OperatorAnd => Either::Left(once(Spanned::new(token.span, Token::OperatorAnd))),
        LogosToken::OperatorOr => Either::Left(once(Spanned::new(token.span, Token::OperatorOr))),
        LogosToken::OperatorXor => Either::Left(once(Spanned::new(token.span, Token::OperatorXor))),
        LogosToken::OperatorNot => Either::Left(once(Spanned::new(token.span, Token::OperatorNot))),
        LogosToken::OperatorAssignment => {
            Either::Left(once(Spanned::new(token.span, Token::OperatorAssignment)))
        }
        LogosToken::Error(e) => {
            Either::Right(Box::new(once(Spanned::new(token.span, Token::Error(e))))
                as Box<dyn Iterator<Item = Spanned<Token>>>)
        }
    }
}

fn map_string_token(token: Spanned<StringToken>) -> impl Iterator<Item = Spanned<Token>> {
    match token.value {
        StringToken::Content(c) => {
            Either::Left(once(Spanned::new(token.span, Token::StringContent(c))))
        }
        StringToken::Escaped(c) => {
            Either::Left(once(Spanned::new(token.span, Token::Character(c))))
        }
        StringToken::BackSlash => {
            Either::Left(once(Spanned::new(token.span, Token::Character('\\'))))
        }
        StringToken::EscapedCurlyOpen => {
            Either::Left(once(Spanned::new(token.span, Token::Character('}'))))
        }
        StringToken::EscapedCurlyClose => {
            Either::Left(once(Spanned::new(token.span, Token::Character('}'))))
        }
        StringToken::FormatStart((expr, format)) => Either::Right(Box::new(
            once(Spanned::new(
                token.span.start..(token.span.start + 1),
                Token::StringFormatOpen,
            ))
            .chain(expr.into_iter().flat_map(map_token))
            .chain(format.into_iter().flat_map(map_string_format_token))
            .chain(once(Spanned::new(
                (token.span.end - 1)..token.span.end,
                Token::StringFormatClose,
            ))),
        )
            as Box<dyn Iterator<Item = Spanned<Token>>>),
        StringToken::Quote => Either::Left(once(Spanned::new(token.span, Token::FormatStringEnd))),
    }
}

fn map_string_format_token(
    token: Spanned<StringFormatToken>,
) -> impl Iterator<Item = Spanned<Token>> {
    match token.value {
        StringFormatToken::FormatEnd => once(Spanned::new(token.span, Token::StringFormatClose)),
        StringFormatToken::FormatPrecision(c) => {
            once(Spanned::new(token.span, Token::StringFormatPrecision(c)))
        }
        StringFormatToken::FormatWidth(c) => {
            once(Spanned::new(token.span, Token::StringFormatWidth(c)))
        }
        StringFormatToken::StringEnd => once(Spanned::new(token.span, Token::StringFormatClose)),
    }
}
