// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Chumsky parser functions for µcad syntax elements

use crate::ast;

use crate::impl_parser;
use crate::parse::*;
use crate::token::Token;

use chumsky::{IterParser, Parser, select_ref};

impl_parser!(ast::Unit => {
    select_ref! {
        Token::Identifier(ident) = e => Self {
            span: e.span(),
            name: ident.as_ref().into()
        },
        Token::Unit(unit) = e => Self {
            span: e.span(),
            name: unit.as_ref().into()
        },
        Token::SigilQuote = e => Self {
            span: e.span(),
            name: r#"""#.into()
        },
    }
    .labelled("unit")
    .boxed()
});

impl_parser!(ast::Literal => {
    use mucad_lang_base::ToCompactString;
    use std::str::FromStr;

    let single_value = select_ref! {
        Token::LiteralFloat(x) = e => {
            match f64::from_str(x) {
                Ok(_) => ast::LiteralKind::Float(ast::FloatLiteral {
                    value: x.to_compact_string(),
                    span: e.span(),
                }),
                Err(err) => ast::LiteralKind::Error(ast::LiteralError {
                    span: e.span(),
                    kind: err.into(),
                })
            }
        },
        Token::LiteralInt(x) = e => {
            match i64::from_str(x) {
                Ok(_) => ast::LiteralKind::Integer(ast::IntegerLiteral {
                value: x.to_compact_string(),
                span: e.span(),
            }),
                Err(err) => ast::LiteralKind::Error(ast::LiteralError {
                    span: e.span(),
                    kind: err.into(),
                })
            }
        },
        Token::LiteralString(content) = e => {
            ast::LiteralKind::String(ast::StringLiteral {
                span: e.span(),
                content: content.as_ref().into(),
            })
        },
        Token::LiteralBool(value) = e => {
            ast::LiteralKind::Bool(ast::BoolLiteral {
                span: e.span(),
                value: *value,
            })
        },
    }
    .boxed();

    single_value
        .then(ast::Unit::parser().or_not())
        .with_extras()
        .try_map_with(|((literal, ty), extras), e| {
            let literal = match (literal, ty) {
                (ast::LiteralKind::Float(float), Some(unit)) => {
                    ast::LiteralKind::Quantity(ast::QuantityLiteral {
                        span: e.span(),
                        value: float.value,
                        unit,
                    })
                }
                (ast::LiteralKind::Integer(int), Some(unit)) => {
                    ast::LiteralKind::Quantity(ast::QuantityLiteral {
                        span: e.span(),
                        value: int.value,
                        unit,
                    })
                }
                (_, Some(_)) => ast::LiteralKind::Error(ast::LiteralError {
                    span: e.span(),
                    kind: ast::LiteralErrorKind::Untypable,
                }),
                (literal, None) => literal,
            };
            Ok(ast::Literal {
                span: e.span(),
                literal,
                extras,
            })
        })
        .labelled("literal")
        .boxed()
});

impl_parser!(ast::Comment => {
    let single_line_comments = select_ref! {
        Token::SingleLineComment(comment) => comment
    }
    .map_with(|line, e| ast::Comment {
        span: e.span(),
        inner: ast::CommentInner::SingleLine(line.to_string()),
    })
    .boxed();
    let multi_line = select_ref! {
        Token::MultiLineComment(comment) = e => ast::Comment {
            span: e.span(),
            inner: ast::CommentInner::MultiLine(comment.to_string())
        }
    };

    single_line_comments
        .or(multi_line)
        .labelled("comment")
        .boxed()
});

impl_parser!(ast::Whitespace => {
    select_ref! {
        Token::Whitespace(s) => Self(s.to_string()),
    }
    .labelled("whitespace")
    .boxed()
});

impl_parser!(ast::LeadingExtras => {
    let whitespace = ast::Whitespace::parser().map(ast::ItemExtra::Whitespace);
    let comment = ast::Comment::parser().map(ast::ItemExtra::Comment);

    comment
        .or(whitespace)
        .repeated()
        .collect::<Vec<_>>()
        .boxed()
        .map(ast::LeadingExtras)
});

impl_parser!(ast::TrailingExtras => {
    let whitespace = ast::Whitespace::parser().map(ast::ItemExtra::Whitespace);
    let comment = ast::Comment::parser().map(ast::ItemExtra::Comment);

    whitespace
        .or(comment)
        .repeated()
        .collect::<Vec<_>>()
        .map(ast::TrailingExtras)
        .boxed()
});
