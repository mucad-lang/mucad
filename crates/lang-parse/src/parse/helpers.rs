// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ast;
use mucad_lang_base::{Span, Spanned};

use crate::parse::{Extra, ParserDefinition, ParserInput, RichError};
use crate::token::Token;
use chumsky::extra::{Full, ParserExtra, SimpleState};
use chumsky::input::Input;
use chumsky::inspector::Inspector;
use chumsky::prelude::*;
use chumsky::{Parser, extra};

/// Ignore tokens, until we hit the end of a pair or nested curly brackets
///
/// Used for error recovery
pub fn ignore_till_matched_curly<'tokens>()
-> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, (), Extra<'tokens>> {
    nested_delimiters(
        Token::SigilOpenCurlyBracket,
        Token::SigilCloseCurlyBracket,
        [
            (
                Token::SigilOpenSquareBracket,
                Token::SigilCloseSquareBracket,
            ),
            (Token::SigilOpenBracket, Token::SigilCloseBracket),
        ],
        |_| (),
    )
    .ignored()
    .boxed()
}

/// Ignore tokens, until we hit the end of a pair or nested brackets
///
/// Used for error recovery
pub fn ignore_till_matched_brackets<'tokens>()
-> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, (), Extra<'tokens>> {
    nested_delimiters(
        Token::SigilOpenBracket,
        Token::SigilCloseBracket,
        [
            (
                Token::SigilOpenSquareBracket,
                Token::SigilCloseSquareBracket,
            ),
            (Token::SigilOpenCurlyBracket, Token::SigilCloseCurlyBracket),
        ],
        |_| (),
    )
    .ignored()
    .boxed()
}

/// Ignore tokens, until we hit a semicolon, without consuming the semicolon
///
/// Used for error recovery
pub fn ignore_till_semi<'tokens>()
-> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, (), Extra<'tokens>> {
    none_of(Token::SigilSemiColon)
        .repeated()
        .at_least(1)
        .then(just(Token::SigilSemiColon).rewind())
        .ignored()
        .boxed()
}

pub fn binop<'tokens, I>(
    params: I,
    tokens: &'static [Token<'static>],
) -> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, ast::Expression, Extra<'tokens>> + Clone
where
    I: Parser<'tokens, ParserInput<'tokens, 'tokens>, ast::Expression, Extra<'tokens>>
        + Clone
        + 'tokens,
{
    use mucad_lang_base::element::BinaryOperator::*;

    params
        .clone()
        .foldl_with(
            ast::Whitespace::parser()
                .or_not()
                .ignore_then(one_of(tokens).map_with(|token, e| Spanned {
                    span: e.span(),
                    value: match token {
                        Token::OperatorAdd => Add,
                        Token::OperatorSubtract => Subtract,
                        Token::OperatorMultiply => Multiply,
                        Token::OperatorDivide => Divide,
                        Token::OperatorUnion => Union,
                        Token::OperatorIntersect => Intersect,
                        Token::OperatorPowerXor => PowerXor,
                        Token::OperatorGreaterThan => GreaterThan,
                        Token::OperatorLessThan => LessThan,
                        Token::OperatorGreaterEqual => GreaterEqual,
                        Token::OperatorLessEqual => LessEqual,
                        Token::OperatorNear => Near,
                        Token::OperatorEqual => Equal,
                        Token::OperatorNotEqual => NotEqual,
                        Token::OperatorAnd => And,
                        Token::OperatorOr => Or,
                        Token::OperatorXor => Xor,
                        _ => unreachable!(),
                    },
                }))
                .then_maybe_whitespace()
                .then(params)
                .repeated(),
            |lhs, (operation, rhs), e| {
                ast::Expression::BinaryOperation(ast::BinaryOperation {
                    span: e.span(),
                    lhs: lhs.into(),
                    op: operation,
                    rhs: rhs.into(),
                })
            },
        )
        .boxed()
}

/// Error recovery parser for places where a single significant token is expected.
///
/// Matches anything but a semicolon or whitespace,
/// if a semicolon or whitespace is encountered, no tokens will be consumed
pub fn recovery_expect_any<'tokens, S, Ctx>()
-> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, (), Full<RichError<'tokens>, S, Ctx>>
+ 'tokens
+ Clone
where
    S: Inspector<'tokens, ParserInput<'tokens, 'tokens>> + Default + Clone + 'static,
    Ctx: 'tokens,
{
    recovery_expect_any_except(&[])
}

/// Same as `recovery_expect_any` but excluding certain tokens
pub fn recovery_expect_any_except<'tokens, S, Ctx>(
    except: &'tokens [Token<'tokens>],
) -> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, (), Full<RichError<'tokens>, S, Ctx>>
+ 'tokens
+ Clone
where
    S: Inspector<'tokens, ParserInput<'tokens, 'tokens>> + Default + Clone + 'static,
    Ctx: 'tokens,
{
    none_of(Token::SigilSemiColon)
        .filter(|t: &Token| t.kind() != "whitespace" && !except.contains(t))
        .ignored()
        .or(one_of(Token::SigilSemiColon)
            .ignored()
            .or(one_of(except).ignored())
            .or(ast::Whitespace::parser().boxed().ignored())
            .rewind())
}

pub trait ParserExt<'src, I, O, E = extra::Default>: Parser<'src, I, O, E>
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
    O: 'src,
{
    fn with_extras(self) -> impl Parser<'src, I, (O, ast::ItemExtras), E> + 'src;

    /// Required a whitespace
    fn then_whitespace(self) -> impl Parser<'src, I, O, E> + 'src;

    /// Allow a whitespace
    fn then_maybe_whitespace(self) -> impl Parser<'src, I, O, E> + 'src;

    fn delimited_with_spanned_error<B, C, U, V, F>(
        self,
        before: B,
        after: C,
        err_map: F,
    ) -> impl Parser<'src, I, O, E>
    where
        B: Parser<'src, I, U, Full<E::Error, SimpleState<Span>, E::Context>>,
        C: Parser<'src, I, V, Full<E::Error, SimpleState<Span>, E::Context>>,
        F: Fn(E::Error, I::Span, I::Span) -> E::Error;
}

impl<'tokens, O, P, S, Ctx>
    ParserExt<'tokens, ParserInput<'tokens, 'tokens>, O, Full<RichError<'tokens>, S, Ctx>> for P
where
    P: Parser<'tokens, ParserInput<'tokens, 'tokens>, O, Full<RichError<'tokens>, S, Ctx>>
        + 'tokens,
    O: 'tokens,
    S: Inspector<'tokens, ParserInput<'tokens, 'tokens>> + Default + Clone + 'static,
    Ctx: 'tokens,
{
    fn with_extras(
        self,
    ) -> impl Parser<
        'tokens,
        ParserInput<'tokens, 'tokens>,
        (O, ast::ItemExtras),
        Full<RichError<'tokens>, S, Ctx>,
    > {
        ast::LeadingExtras::parser()
            .then(self)
            .then(ast::TrailingExtras::parser())
            .map(|((leading, res), trailing)| (res, ast::ItemExtras { leading, trailing }))
            .boxed()
    }

    fn then_whitespace(
        self,
    ) -> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, O, Full<RichError<'tokens>, S, Ctx>>
    {
        self.then_ignore(ast::Whitespace::parser())
    }

    fn then_maybe_whitespace(
        self,
    ) -> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, O, Full<RichError<'tokens>, S, Ctx>>
    {
        self.then_ignore(ast::Whitespace::parser().or_not())
    }

    fn delimited_with_spanned_error<B, C, U, V, F>(
        self,
        before: B,
        after: C,
        err_map: F,
    ) -> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, O, Full<RichError<'tokens>, S, Ctx>>
    where
        B: Parser<
                'tokens,
                ParserInput<'tokens, 'tokens>,
                U,
                Full<RichError<'tokens>, SimpleState<Span>, Ctx>,
            >,
        C: Parser<
                'tokens,
                ParserInput<'tokens, 'tokens>,
                V,
                Full<RichError<'tokens>, SimpleState<Span>, Ctx>,
            >,
        F: Fn(RichError<'tokens>, Span, Span) -> RichError<'tokens>,
    {
        before
            .map_with(|_, e| *e.state() = e.span().into())
            .then(self.with_state(S::default()))
            .then(
                after.map_err_with_state(move |e, span: Span, state| {
                    err_map(e, state.0.clone(), span)
                }),
            )
            .map(|((_, res), _)| res)
            .with_state(SimpleState(0..0))
    }
}
