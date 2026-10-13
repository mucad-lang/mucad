// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod diag;
mod helpers;
mod parse_context;
pub mod parsers;

pub use diag::{ParseError, ParseErrorKind, ParseInfo, ParseIssue, ParseWarning};
pub use parse_context::ParseContext;

use crate::{
    ast, lex,
    parse::{
        diag::{Rich, RichError},
        helpers::*,
    },
    token::Token,
};

use chumsky::{
    Parser, extra,
    input::{Input, MappedInput},
    inspector::Inspector,
    prelude::*,
    select_ref,
};

use std::str::FromStr;

use microcad_lang_base::{Span, Spanned};

/// Extra error.
pub type Extra<'tokens> = extra::Err<RichError<'tokens>>;

pub type InputMap<'input, 'token> =
    fn(&'input Spanned<Token<'token>>) -> (&'input Token<'token>, &'input Span);

pub type ParserInput<'input, 'token> = MappedInput<
    'input,
    Token<'token>,
    Span,
    &'input [Spanned<Token<'token>>],
    InputMap<'input, 'token>,
>;

/// Alias for parser input type
pub type PInput<'a> = ParserInput<'a, 'a>;

/// Alias for parser error type
pub type PError<'a, S, Ctx> = extra::Full<RichError<'a>, S, Ctx>;

/// Alias for Inspector type (as a trait bound shorthand)
pub trait PInspector<'a>: Inspector<'a, PInput<'a>> + Default + Clone + 'static {}

impl<'a, T> PInspector<'a> for T where T: Inspector<'a, PInput<'a>> + Default + Clone + 'static {}

pub trait ParserDefinition: Sized {
    fn parser<'tokens, S, Ctx>()
    -> impl Parser<'tokens, PInput<'tokens>, Self, PError<'tokens, S, Ctx>>
    where
        S: PInspector<'tokens>,
        Ctx: 'tokens;
}

/// implement a parser for a specific AST node type.
#[macro_export]
macro_rules! impl_parser {
    ($target_struct:ty => $body:expr) => {
        impl $crate::parse::parsers::ParserDefinition for $target_struct {
            fn parser<'tokens, S, Ctx>() -> impl ::chumsky::Parser<
                'tokens,
                $crate::parse::parsers::PInput<'tokens>,
                Self,
                $crate::parse::parsers::PError<'tokens, S, Ctx>,
            >
            where
                S: $crate::parse::parsers::PInspector<'tokens>,
                Ctx: 'tokens,
            {
                $body
            }
        }
    };
}

/// Get parser input from tokens
pub fn input<'input, 'tokens>(
    input: &'input [Spanned<Token<'tokens>>],
) -> ParserInput<'input, 'tokens> {
    fn map_token_input<'a, 'token>(
        spanned: &'a Spanned<Token<'token>>,
    ) -> (&'a Token<'token>, &'a Span) {
        (&spanned.value, &spanned.span)
    }

    let end = input.last().map(|t| t.span.end).unwrap_or_default();
    Input::map(input, end..end, map_token_input)
}

/// Build an abstract syntax tree from a list of tokens
pub fn parse<'tokens>(
    tokens: &'tokens [Spanned<Token<'tokens>>],
    context: &ParseContext,
) -> Result<ast::Source, Vec<ParseError>> {
    parser()
        .parse(input(tokens))
        .into_result()
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|error| ParseError::new(error, context))
                .collect::<Vec<_>>()
        })
}

const STRUCTURAL_TOKENS: &[Token] = &[
    Token::SigilOpenCurlyBracket,
    Token::SigilCloseCurlyBracket,
    Token::SigilOpenBracket,
    Token::SigilCloseBracket,
    Token::SigilOpenSquareBracket,
    Token::SigilCloseSquareBracket,
    Token::SigilSemiColon,
];

fn parser<'tokens>()
-> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, ast::Source, Extra<'tokens>> {
    use crate::ast::Dummy;

    let mut statement_list_parser = Recursive::declare();
    let mut statement_parser = Recursive::declare();
    let mut expression_parser = Recursive::declare();
    let mut type_parser = Recursive::declare();
    let mut outer_attribute_parser = Recursive::declare();
    let mut if_inner = Recursive::declare();

    let semi_recovery = none_of(Token::SigilSemiColon).repeated().ignored();

    let ws = ast::Whitespace::parser().boxed();

    let reserved_keyword = select_ref! {
        token @ (
            Token::KeywordPlugin |
            Token::KeywordAssembly |
            Token::KeywordMaterial |
            Token::KeywordUnit |
            Token::KeywordEnum |
            Token::KeywordStruct |
            Token::KeywordMatch |
            Token::KeywordType |
            Token::KeywordExtern
        ) => token.kind(),
    }
    .boxed();
    let keyword = select_ref! {
        token @ (
            Token::KeywordMod |
            Token::KeywordPart |
            Token::KeywordSketch |
            Token::KeywordOp |
            Token::KeywordFn |
            Token::KeywordIf |
            Token::KeywordElse |
            Token::KeywordUse |
            Token::KeywordAs |
            Token::KeywordReturn |
            Token::KeywordPub |
            Token::KeywordConst |
            Token::KeywordProp |
            Token::KeywordInit |
            Token::KeywordSuper
        ) => token.kind(),
    }
    .boxed();

    statement_list_parser.define({
        let trailing_expr = outer_attribute_parser
            .clone()
            .then(expression_parser.clone())
            .with_extras()
            .map_with(|((attr, expr), extras), e| ast::ExpressionStatement {
                span: e.span(),
                extras,
                attr,
                expr,
            })
            .map(Box::new)
            .or_not()
            .boxed();

        ws.clone()
            .or_not()
            .ignore_then(statement_parser.clone())
            .then(ast::TrailingExtras::parser())
            .repeated()
            .collect::<Vec<(ast::Statement, ast::TrailingExtras)>>()
            .then(trailing_expr)
            .with_extras()
            .map_with(|((statements, tail), extras), e| ast::StatementList {
                span: e.span(),
                extras,
                statements,
                tail,
            })
            .boxed()
    });

    let block_recovery =
        ignore_till_matched_curly().map_with(|_, e| ast::StatementList::dummy(e.span()));

    let body = ws
        .clone()
        .or_not()
        .ignore_then(statement_list_parser.clone().delimited_with_spanned_error(
            just(Token::SigilOpenCurlyBracket),
            just(Token::SigilCloseCurlyBracket),
            |err: RichError, open, end| {
                Rich::custom(
                    err.span().clone(),
                    ParseErrorKind::UnclosedBracket {
                        open,
                        end,
                        kind: "code block",
                        close_token: Token::SigilCloseCurlyBracket,
                    },
                )
            },
        ))
        .recover_with(via_parser(block_recovery))
        .map_with(|statements, e| ast::Body {
            span: e.span(),
            statements,
        })
        .boxed();

    let identifier_parser = select_ref! { Token::Identifier(ident) = e => ast::Identifier {
        span: e.span(),
        name: ident.as_ref().into(),
    } }
    .or(reserved_keyword
        .clone()
        .validate(|kind, e, emitter| {
            emitter.emit(Rich::custom(
                e.span(),
                ParseErrorKind::ReservedKeywordAsIdentifier(kind),
            ));
            kind
        })
        .map_with(|kind, e| ast::Identifier {
            span: e.span(),
            name: kind.into(),
        }))
    .or(keyword
        .validate(|kind, e, emitter| {
            emitter.emit(Rich::custom(
                e.span(),
                ParseErrorKind::KeywordAsIdentifier(kind),
            ));
            kind
        })
        .map_with(|kind, e| ast::Identifier {
            span: e.span(),
            name: kind.into(),
        }))
    .labelled("identifier")
    .boxed();

    // `super` is a keyword, which is only allowed as leading parts of a path,
    // e.g. `super::super::geo::Circle`. It becomes a part named `super`.
    let super_parts = just(Token::KeywordSuper)
        .map_with(|token, e| ast::Identifier {
            span: e.span(),
            name: token.kind().into(),
        })
        .then_ignore(just(Token::SigilDoubleColon))
        .repeated()
        .collect::<Vec<_>>()
        .boxed();

    let symbol_path = select_ref! {
        Token::SigilDoubleColon = e => e.span()
    }
    .or_not()
    .then(
        super_parts
            .clone()
            .then(
                identifier_parser
                    .clone()
                    .separated_by(just(Token::SigilDoubleColon))
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .map(|(mut parts, identifiers)| {
                parts.extend(identifiers);
                parts
            }),
    )
    .with_extras()
    .map_with(|((prefix, parts), extras), e| ast::SymbolPath {
        span: e.span(),
        prefix,
        parts,
        extras,
    })
    .labelled("qualified name")
    .boxed();

    let unit = ast::Unit::parser().boxed();

    type_parser.define({
        let single = select_ref! {
            Token::Identifier(ident) = e => ast::SingleType {
                span: e.span(),
                name: ident.as_ref().into()
            },
        }
        .map(ast::Type::Single)
        .labelled("single type")
        .boxed();

        let list = ws
            .clone()
            .or_not()
            .ignore_then(type_parser.clone())
            .then_maybe_whitespace()
            .delimited_by(
                just(Token::SigilOpenSquareBracket),
                just(Token::SigilCloseSquareBracket),
            )
            .map_with(|inner, e| {
                ast::Type::List(ast::ListType {
                    span: e.span(),
                    inner: Box::new(inner),
                })
            })
            .labelled("list type")
            .boxed();

        let tuple = ws
            .clone()
            .or_not()
            .ignore_then(identifier_parser.clone())
            .then_ignore(just(Token::SigilColon))
            .or_not()
            .then_maybe_whitespace()
            .then(type_parser.clone())
            .then_maybe_whitespace()
            .separated_by(just(Token::SigilComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .then_maybe_whitespace()
            .delimited_with_spanned_error(
                just(Token::SigilOpenBracket),
                just(Token::SigilCloseBracket),
                |err: RichError, open, end| {
                    Rich::custom(
                        err.span().clone(),
                        ParseErrorKind::UnclosedBracket {
                            open,
                            end,
                            kind: "tuple type",
                            close_token: Token::SigilCloseBracket,
                        },
                    )
                },
            )
            .map_with(|inner, e| {
                ast::Type::Tuple(ast::TupleType {
                    span: e.span(),
                    inner,
                })
            })
            .labelled("tuple type")
            .boxed();

        single.or(list).or(tuple).labelled("type").boxed()
    });

    let unary_operator_parser = select_ref! {
        Token::OperatorSubtract = e => Spanned { span: e.span(), value: ast::UnaryOperator::Minus },
        Token::OperatorAdd = e => Spanned { span: e.span(), value: ast::UnaryOperator::Plus },
        Token::OperatorNot = e => Spanned { span: e.span(), value: ast::UnaryOperator::Not },
    }
    .labelled("unary operator")
    .boxed();

    // Parse for a DocBlock, starting with `///`
    let doc_block = select_ref! {
        Token::DocComment(comment) => comment.to_string(),
    }
    .then_whitespace()
    .repeated()
    .collect::<Vec<_>>()
    .map_with(|lines, e| ast::DocBlock {
        span: e.span(),
        lines,
    })
    .labelled("doc block")
    .boxed();

    let tuple_recovery = nested_delimiters(
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
    .map_with(|_, e| {
        (
            vec![ast::TupleItem::dummy(e.span())],
            ast::ItemExtras::default(),
        )
    });

    let tuple_body = identifier_parser
        .clone()
        .then_maybe_whitespace()
        .then_ignore(just(Token::OperatorAssignment).then_maybe_whitespace())
        .or_not()
        .then(expression_parser.clone())
        .with_extras()
        .map_with(|((id, expr), extras), e| ast::TupleItem {
            span: e.span(),
            extras,
            id,
            expr,
        })
        .then_maybe_whitespace()
        .separated_by(just(Token::SigilComma).then_maybe_whitespace())
        .allow_trailing()
        .collect::<Vec<_>>()
        .boxed();

    let call_inner = symbol_path
        .clone()
        .then(
            tuple_body
                .clone()
                .with_extras()
                .map_with(|(arguments, extras), e| ast::ArgumentList {
                    span: e.span(),
                    extras,
                    arguments: arguments
                        .into_iter()
                        .map(|item| match item.id {
                            Some(id) => ast::Argument::Named(ast::NamedArgument {
                                span: item.span,
                                extras: item.extras,
                                id,
                                expr: item.expr,
                            }),
                            None => ast::Argument::Unnamed(ast::UnnamedArgument {
                                span: item.span,
                                extras: item.extras,
                                expr: item.expr,
                            }),
                        })
                        .collect::<Vec<_>>(),
                })
                .labelled("function arguments")
                .delimited_with_spanned_error(
                    just(Token::SigilOpenBracket),
                    just(Token::SigilCloseBracket),
                    |err: RichError, open, end| {
                        Rich::custom(
                            err.span().clone(),
                            ParseErrorKind::UnclosedBracket {
                                open,
                                end,
                                kind: "function arguments",
                                close_token: Token::SigilCloseBracket,
                            },
                        )
                    },
                )
                .recover_with(via_parser(
                    tuple_recovery
                        .clone()
                        .map_with(|_, e| ast::ArgumentList::dummy(e.span())),
                )),
        )
        .with_extras()
        .map_with(|((path, arguments), extras), e| ast::Call {
            span: e.span(),
            extras,
            path,
            arguments,
        })
        .boxed();

    statement_parser.define({
        let visibility = select_ref! {
            Token::KeywordPub => ast::def::Visibility::Public,
        }
        .map_with(|vis, e| Spanned::new(e.span(), vis))
        .labelled("visibility");

        let expression = outer_attribute_parser
            .clone()
            .then(expression_parser.clone())
            .with_extras()
            .map_with(|((attr, expr), extras), e| ast::ExpressionStatement {
                span: e.span(), // FIXME: This should only return the span of attributes and expression
                extras,
                attr,
                expr,
            })
            .map(ast::Statement::Expression)
            .boxed();

        let expression_without_semi = outer_attribute_parser
            .clone()
            .then(expression_parser.clone().try_map(|expr, span| {
                if expr.is_also_statement() {
                    Ok(expr)
                } else {
                    Err(Rich::custom(
                        span.clone(),
                        ParseErrorKind::ExpressionMissingSemicolon { span },
                    ))
                }
            }))
            .with_extras()
            .map_with(|((attr, expr), extras), e| ast::ExpressionStatement {
                span: e.span(), // FIXME: This should only return the span of attributes and expression
                extras,
                attr,
                expr,
            })
            .map(ast::Statement::Expression)
            .boxed();

        let local_assignment_inner = outer_attribute_parser
            .clone()
            .then(identifier_parser.clone())
            .then_maybe_whitespace()
            .then(
                just(Token::SigilColon)
                    .then_maybe_whitespace()
                    .ignore_then(type_parser.clone())
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .then_ignore(just(Token::OperatorAssignment))
            .then_maybe_whitespace()
            .then(
                expression_parser.clone().recover_with(via_parser(
                    semi_recovery
                        .clone()
                        .map_with(|_, e| ast::Expression::Error(e.span())),
                )),
            )
            .with_extras()
            .map_with(
                |((((attr, id), ty), expr), extras), e| ast::LocalAssignment {
                    span: e.span(),
                    extras,
                    attr,
                    id,
                    expr: Box::new(expr),
                    ty,
                },
            )
            .boxed();

        let local_assignment = local_assignment_inner
            .clone()
            .map(ast::Statement::LocalAssignment)
            .labelled("local assignment");

        let const_assignment_inner = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(visibility.then_whitespace().or_not())
            .then(just(Token::KeywordConst).map_with(|_, e| e.span()))
            .then_maybe_whitespace()
            .then(identifier_parser.clone())
            .then_maybe_whitespace()
            .then(
                just(Token::SigilColon)
                    .then_maybe_whitespace()
                    .ignore_then(type_parser.clone())
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .then_ignore(just(Token::OperatorAssignment))
            .then_maybe_whitespace()
            .then(
                expression_parser.clone().recover_with(via_parser(
                    semi_recovery
                        .clone()
                        .map_with(|_, e| ast::Expression::Error(e.span())),
                )),
            )
            .with_extras()
            .map_with(
                |(((((((doc, attr), vis), keyword_span), id), ty), value), extras), e| {
                    ast::def::Constant {
                        span: e.span(),
                        keyword_span,
                        extras,
                        doc,
                        attr,
                        vis,
                        id,
                        expr: Box::new(value),
                        ty,
                    }
                },
            )
            .boxed();

        let const_assignment = const_assignment_inner
            .clone()
            .map(ast::Statement::Const)
            .labelled("const assignment");

        // A pub assignment without the `const` keyword will eventually become a const assignment
        let pub_assignment_inner = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(just(Token::KeywordPub).map_with(|_, e| e.span()))
            .then_maybe_whitespace()
            .then(identifier_parser.clone())
            .then_maybe_whitespace()
            .then(
                just(Token::SigilColon)
                    .then_maybe_whitespace()
                    .ignore_then(type_parser.clone())
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .then_ignore(just(Token::OperatorAssignment))
            .then_maybe_whitespace()
            .then(
                expression_parser.clone().recover_with(via_parser(
                    semi_recovery
                        .clone()
                        .map_with(|_, e| ast::Expression::Error(e.span())),
                )),
            )
            .with_extras()
            .map_with(
                |((((((doc, attr), keyword_span), id), ty), expr), extras), e| ast::def::Constant {
                    span: e.span(),
                    vis: Some(Spanned::new(
                        keyword_span.clone(),
                        ast::def::Visibility::Public,
                    )),
                    keyword_span,
                    extras,
                    doc,
                    attr,
                    id,
                    expr: Box::new(expr),
                    ty,
                },
            )
            .boxed();

        let pub_assignment = pub_assignment_inner
            .clone()
            .map(ast::Statement::Const)
            .labelled("pub const assignment");

        let property_assignment_inner = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(just(Token::KeywordProp).map_with(|_, e| e.span()))
            .then_maybe_whitespace()
            .then(identifier_parser.clone())
            .then_maybe_whitespace()
            .then(
                just(Token::SigilColon)
                    .then_maybe_whitespace()
                    .ignore_then(type_parser.clone())
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .then_ignore(just(Token::OperatorAssignment))
            .then_maybe_whitespace()
            .then(
                expression_parser.clone().recover_with(via_parser(
                    semi_recovery
                        .clone()
                        .map_with(|_, e| ast::Expression::Error(e.span())),
                )),
            )
            .with_extras()
            .map_with(
                |((((((doc, attr), keyword_span), id), ty), value), extras), e| {
                    ast::PropertyAssignment {
                        span: e.span(),
                        keyword_span,
                        extras,
                        doc,
                        attr,
                        id,
                        value: Box::new(value),
                        ty,
                    }
                },
            )
            .boxed();

        let property_assignment = property_assignment_inner
            .clone()
            .map(ast::Statement::Property)
            .labelled("property assignment");

        let attribute_command = local_assignment_inner
            .clone()
            .map(ast::AttributeCommand::Assignment)
            .or(call_inner.clone().map(ast::AttributeCommand::Call))
            .or(identifier_parser.clone().map(ast::AttributeCommand::Ident));

        let attribute_inner = attribute_command
            .separated_by(just(Token::SigilComma).then_maybe_whitespace())
            .at_least(1)
            .collect::<Vec<_>>()
            .then_maybe_whitespace()
            .delimited_by(
                just(Token::SigilOpenSquareBracket),
                just(Token::SigilCloseSquareBracket),
            )
            .boxed();

        outer_attribute_parser.define({
            just(Token::SigilHash)
                .ignore_then(attribute_inner.clone())
                .then_whitespace()
                .with_extras()
                .map_with(|(commands, extras), e| ast::Attribute {
                    span: e.span(),
                    is_inner: false,
                    extras,
                    commands,
                })
                .labelled("attribute")
                .repeated()
                .collect::<Vec<ast::Attribute>>()
                .map(ast::Attributes)
                .boxed()
        });

        let parameter_list_inner = ws
            .clone()
            .or_not()
            .ignore_then(doc_block.clone())
            .then(outer_attribute_parser.clone())
            .then(identifier_parser.clone())
            .then_maybe_whitespace()
            .then(
                just(Token::SigilColon)
                    .then_maybe_whitespace()
                    .ignore_then(
                        type_parser.clone().recover_with(via_parser(
                            recovery_expect_any_except(&[
                                Token::SigilComma,
                                Token::OperatorAssignment,
                                Token::SigilCloseBracket,
                            ])
                            .map_with(|_, e| ast::Type::dummy(e.span())),
                        )),
                    )
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .then(
                just(Token::OperatorAssignment)
                    .then_maybe_whitespace()
                    .ignore_then(
                        expression_parser.clone().recover_with(via_parser(
                            recovery_expect_any_except(&[
                                Token::SigilComma,
                                Token::SigilCloseBracket,
                            ])
                            .map_with(|_, e| ast::Expression::Error(e.span())),
                        )),
                    )
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .with_extras()
            .map_with(
                |(((((doc, attr), id), ty), default), extras), e| ast::Parameter {
                    span: e.span(),
                    doc,
                    attr,
                    extras,
                    id,
                    ty,
                    default,
                },
            )
            .separated_by(just(Token::SigilComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .boxed();

        let parameter_list = parameter_list_inner
            .then_maybe_whitespace()
            .with_extras()
            .map_with(|(parameters, extras), e| ast::ParameterList {
                span: e.span(),
                extras,
                parameters,
            })
            .delimited_with_spanned_error(
                just(Token::SigilOpenBracket),
                just(Token::SigilCloseBracket),
                |err: RichError, open, end| {
                    Rich::custom(
                        err.span().clone(),
                        ParseErrorKind::UnclosedBracket {
                            open,
                            end,
                            kind: "function parameters",
                            close_token: Token::SigilCloseBracket,
                        },
                    )
                },
            )
            .recover_with(via_parser(
                ignore_till_matched_brackets()
                    .or(none_of(STRUCTURAL_TOKENS).repeated())
                    .map_with(|_, e| ast::ParameterList::dummy(e.span())),
            ))
            .boxed();

        let inline_module = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(visibility.then_whitespace().or_not())
            .then(just(Token::KeywordMod).map_with(|_, e| e.span()))
            .then_whitespace()
            .then(
                identifier_parser.clone().recover_with(via_parser(
                    recovery_expect_any_except(&[Token::SigilOpenCurlyBracket])
                        .map_with(|_, e| ast::Identifier::dummy(e.span())),
                )),
            )
            .then_maybe_whitespace()
            .then(body.clone())
            .with_extras()
            .map_with(
                |((((((doc, attr), vis), keyword_span), id), body), extras), e| {
                    ast::Statement::InlineModule(ast::def::InlineModule {
                        span: e.span(),
                        keyword_span,
                        extras,
                        doc,
                        attr,
                        vis,
                        id,
                        body,
                    })
                },
            )
            .boxed();

        let file_module = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(visibility.then_whitespace().or_not())
            .then(just(Token::KeywordMod).map_with(|_, e| e.span()))
            .then_whitespace()
            .then(
                identifier_parser.clone().recover_with(via_parser(
                    recovery_expect_any_except(&[Token::SigilOpenCurlyBracket])
                        .map_with(|_, e| ast::Identifier::dummy(e.span())),
                )),
            )
            .with_extras()
            .map_with(|(((((doc, attr), vis), keyword_span), id), extras), e| {
                ast::Statement::FileModule(ast::def::FileModule {
                    span: e.span(),
                    keyword_span,
                    extras,
                    doc,
                    attr,
                    vis,
                    id,
                })
            })
            .boxed();

        let use_part = identifier_parser
            .clone()
            .map(ast::def::UseStatementPart::Identifier)
            .or(just(Token::OperatorMultiply)
                .map_with(|_, e| ast::def::UseStatementPart::Glob(e.span())))
            .recover_with(via_parser(
                recovery_expect_any_except(&[Token::SigilDoubleColon])
                    .map_with(|_, e| ast::def::UseStatementPart::Error(e.span())),
            ))
            .boxed();

        let use_name = select_ref! {
            Token::SigilDoubleColon = e => e.span()
        }
        .or_not()
        .then(
            super_parts
                .clone()
                .then(
                    use_part
                        .separated_by(just(Token::SigilDoubleColon))
                        .at_least(1)
                        .collect::<Vec<_>>(),
                )
                .map(|(supers, mut parts)| {
                    parts.splice(
                        0..0,
                        supers
                            .into_iter()
                            .map(ast::def::UseStatementPart::Identifier),
                    );
                    parts
                }),
        )
        .with_extras()
        .map_with(|((prefix, parts), extras), e| ast::def::UseName {
            span: e.span(),
            prefix,
            extras,
            parts,
        })
        .boxed();

        let use_statement = outer_attribute_parser
            .clone()
            .then(visibility.then_whitespace().or_not())
            .then(just(Token::KeywordUse).map_with(|_, e| e.span()))
            .then_whitespace()
            .then(use_name)
            .then(
                select_ref! {
                    Token::KeywordAs => (),
                }
                .then_whitespace()
                .ignore_then(identifier_parser.clone().recover_with(via_parser(
                    recovery_expect_any().map_with(|_, e| ast::Identifier::dummy(e.span())),
                )))
                .or_not(),
            )
            .with_extras()
            .map_with(
                |(((((attr, vis), keyword_span), name), use_as), extras), e| {
                    ast::Statement::Use(ast::def::Use {
                        span: e.span(),
                        attr,
                        vis,
                        keyword_span,
                        extras,
                        name,
                        use_as,
                    })
                },
            )
            .boxed();

        let workbench_kind = select_ref! {
            Token::KeywordSketch => ast::def::WorkbenchKind::Sketch,
            Token::KeywordPart => ast::def::WorkbenchKind::Part,
            Token::KeywordOp => ast::def::WorkbenchKind::Op,
        }
        .boxed();

        let init = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(just(Token::KeywordInit).map_with(|_, e| e.span()))
            .then_maybe_whitespace()
            .then(parameter_list.clone())
            .then(body.clone())
            .with_extras()
            .map_with(
                |(((((doc, attr), keyword_span), parameters), body), extras), e| {
                    ast::Statement::Init(ast::Init {
                        span: e.span(),
                        keyword_span,
                        extras,
                        doc,
                        attr,
                        parameters,
                        body,
                    })
                },
            )
            .boxed();
        let workbench = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(visibility.then_whitespace().or_not())
            .then(workbench_kind.map_with(|kind, e| (kind, e.span())))
            .then_whitespace()
            .then(
                identifier_parser.clone().recover_with(via_parser(
                    recovery_expect_any_except(&[
                        Token::SigilOpenCurlyBracket,
                        Token::SigilOpenBracket,
                    ])
                    .map_with(|_, e| ast::Identifier::dummy(e.span())),
                )),
            )
            .then_maybe_whitespace()
            .then(parameter_list.clone())
            .then_maybe_whitespace()
            .then(body.clone())
            .with_extras()
            .map_with(
                |(
                    ((((((doc, attr), vis), (kind, keyword_span)), id), parameters), body),
                    extras,
                ),
                 e| {
                    ast::Statement::Workbench(ast::def::Workbench {
                        span: e.span(),
                        keyword_span,
                        extras,
                        kind,
                        doc,
                        attr,
                        vis,
                        id,
                        parameters,
                        body,
                    })
                },
            )
            .boxed();

        let return_statement = just(Token::KeywordReturn)
            .map_with(|_, e| e.span())
            .then_maybe_whitespace()
            .then(expression_parser.clone().or_not())
            .with_extras()
            .map_with(|((keyword_span, expr), extras), e| {
                ast::Statement::Return(ast::Return {
                    span: e.span(),
                    keyword_span,
                    extras,
                    expr,
                })
            })
            .boxed();

        let function = doc_block
            .clone()
            .then(outer_attribute_parser.clone())
            .then(visibility.then_whitespace().or_not())
            .then(just(Token::KeywordFn).map_with(|_, e| e.span()))
            .then_whitespace()
            .then(
                identifier_parser.clone().recover_with(via_parser(
                    recovery_expect_any_except(&[
                        Token::SigilOpenCurlyBracket,
                        Token::SigilOpenBracket,
                    ])
                    .map_with(|_, e| ast::Identifier::dummy(e.span())),
                )),
            )
            .then_maybe_whitespace()
            .then(parameter_list.clone())
            .then_maybe_whitespace()
            .then(
                just(Token::SigilSingleArrow)
                    .then_maybe_whitespace()
                    .ignore_then(type_parser.clone())
                    .then_maybe_whitespace()
                    .or_not(),
            )
            .then(body.clone())
            .with_extras()
            .map_with(
                |(
                    (((((((doc, attr), vis), keyword_span), id), parameters), return_type), body),
                    extras,
                ),
                 e| {
                    ast::Statement::Function(ast::def::Function {
                        span: e.span(),
                        keyword_span,
                        extras,
                        doc,
                        attr,
                        vis,
                        id,
                        parameters,
                        return_type,
                        body,
                    })
                },
            )
            .boxed();

        let inner_doc_comment = select_ref! {
            Token::InnerDocComment(comment) => comment.to_string(),
        }
        .labelled("inner doc-block")
        .map_with(|line, e| {
            ast::Statement::InnerDocComment(ast::InnerDocComment {
                span: e.span(),
                line,
            })
        })
        .boxed();

        let inner_attribute = just(Token::SigilHash)
            .ignore_then(just(Token::OperatorNot))
            .ignore_then(attribute_inner)
            .with_extras()
            .map_with(|(commands, extras), e| ast::Attribute {
                span: e.span(),
                is_inner: true,
                extras,
                commands,
            })
            .labelled("inner attribute")
            .map(ast::Statement::InnerAttribute)
            .boxed();

        let not_assignment = ws
            .clone()
            .or_not()
            .then(none_of([
                Token::OperatorAssignment,
                Token::SigilDoubleColon,
            ]))
            .ignored()
            .or(just(Token::SigilSemiColon).ignored())
            .rewind()
            .boxed();

        let reserved_keyword_statement = reserved_keyword
            .clone()
            .then_ignore(not_assignment.clone())
            .try_map_with(|kind, e| {
                Err::<(), _>(Rich::custom(
                    e.span(),
                    ParseErrorKind::ReservedKeyword(kind),
                ))
            })
            .ignored()
            .recover_with(via_parser(
                reserved_keyword
                    .then_ignore(not_assignment)
                    .clone()
                    .ignore_then(
                        none_of(STRUCTURAL_TOKENS)
                            .repeated()
                            .then_ignore(ignore_till_matched_curly())
                            .or(ignore_till_semi().then_ignore(just(Token::SigilSemiColon))),
                    ),
            ))
            .map_with(|_, e| ast::Statement::Error(e.span()))
            .boxed();

        let with_semi = return_statement
            .or(use_statement)
            .or(const_assignment)
            .or(pub_assignment)
            .or(file_module)
            .or(property_assignment)
            .or(local_assignment)
            .or(expression)
            .then_ignore(
                just(Token::SigilSemiColon)
                    .labelled("semicolon")
                    .ignored()
                    .recover_with(via_parser(
                        none_of(STRUCTURAL_TOKENS)
                            .repeated()
                            .then_ignore(just(Token::SigilSemiColon)),
                    )),
            )
            .boxed();

        let without_semi = function
            .or(inner_doc_comment)
            .or(inner_attribute)
            .or(init)
            .or(workbench)
            .or(inline_module)
            .or(expression_without_semi)
            .boxed();

        with_semi
            .or(reserved_keyword_statement)
            .or(without_semi)
            .boxed()
            .labelled("statement")
    });

    expression_parser.define({
        let unclosed_string = select_ref! {
            Token::Error(lex::LexerError::UnclosedString(_)) => (),
        }
        .ignore_then(
            semi_recovery
                .clone()
                .try_map_with(|_, e| {
                    let span: Span = e.span();
                    Err::<ast::Expression, _>(Rich::custom(
                        (span.start - 1)..span.end,
                        ParseErrorKind::UnterminatedString,
                    ))
                })
                .recover_with(via_parser(
                    semi_recovery
                        .clone()
                        .map_with(|_, e| ast::Expression::Error(e.span())),
                )),
        )
        .labelled("unclosed string")
        .boxed();

        let literal = ast::Literal::parser()
            .map(ast::Expression::Literal)
            .labelled("literal")
            .boxed()
            .or(unclosed_string);

        let marker = just(Token::SigilAt)
            .ignore_then(identifier_parser.clone())
            .map(ast::Expression::Marker)
            .labelled("marker")
            .boxed();

        let string_content_part = select_ref! {
            Token::StringContent(content) = e => ast::StringPart::Content(ast::StringLiteral {
                span: e.span(),
                content: content.as_ref().into(),
            }),
            Token::Character(char) = e => ast::StringPart::Char(ast::StringCharacter {
                span: e.span(),
                character: *char,
            }),
        }
        .labelled("string content")
        .boxed();

        let format_precision = select_ref!(
            Token::StringFormatPrecision(precision) = e => {
                u32::from_str(&precision[1..]).map_err(|err| (err, e.span()))
            }
        );
        let format_width = select_ref!(
            Token::StringFormatWidth(width) = e => {
                u32::from_str(&width[1..]).map_err(|err| (err, e.span()))
            }
        );
        let format_spec = format_width
            .or_not()
            .then(format_precision.or_not())
            .map_with(|(width, precision), e| ast::StringFormatSpecification {
                span: e.span(),
                width,
                precision,
            })
            .labelled("string format specification")
            .boxed();

        let string_format_part = expression_parser
            .clone()
            .then(format_spec)
            .with_extras()
            .delimited_by(
                just(Token::StringFormatOpen),
                just(Token::StringFormatClose),
            )
            .map_with(
                |((expression, specification), extras), e| ast::StringExpression {
                    span: e.span(),
                    extras,
                    expr: Box::new(expression),
                    specification: Box::new(specification),
                },
            )
            .map(ast::StringPart::Expression)
            .labelled("string format expression")
            .boxed();
        let string_part = string_content_part
            .or(string_format_part)
            .labelled("format string content");

        let string_format = string_part
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::FormatStringStart), just(Token::FormatStringEnd))
            .with_extras()
            .map_with(|(parts, extras), e| ast::FormatString {
                span: e.span(),
                extras,
                parts,
            })
            .map(ast::Expression::String)
            .boxed();

        let tuple = ws
            .clone()
            .or_not()
            .ignore_then(tuple_body.clone())
            .with_extras()
            .delimited_with_spanned_error(
                just(Token::SigilOpenBracket),
                just(Token::SigilCloseBracket),
                |err: RichError, open, end| {
                    Rich::custom(
                        err.span().clone(),
                        ParseErrorKind::UnclosedBracket {
                            open,
                            end,
                            kind: "tuple",
                            close_token: Token::SigilCloseBracket,
                        },
                    )
                },
            )
            .map_with(|(values, extras), e| {
                ast::Expression::Tuple(ast::TupleExpression {
                    span: e.span(),
                    extras,
                    values,
                })
            })
            .labelled("tuple");

        let bracketed = expression_parser
            .clone()
            .then_maybe_whitespace()
            .delimited_with_spanned_error(
                just(Token::SigilOpenBracket).then_maybe_whitespace(),
                just(Token::SigilCloseBracket),
                |err: RichError, open, end| {
                    Rich::custom(
                        err.span().clone(),
                        ParseErrorKind::UnclosedBracket {
                            open,
                            end,
                            kind: "bracketed expression",
                            close_token: Token::SigilCloseBracket,
                        },
                    )
                },
            )
            .map_with(|expr, e| ast::Expression::Bracketed(Box::new(expr), e.span()))
            .boxed();

        let list_item = expression_parser
            .clone()
            .with_extras()
            .map_with(|(expr, extras), e| ast::ListItem {
                span: e.span(),
                extras,
                expr,
            })
            .boxed();

        let range = list_item
            .clone()
            .then_ignore(just(Token::SigilDoubleDot))
            .then(list_item.clone())
            .with_extras()
            .delimited_by(
                just(Token::SigilOpenSquareBracket).then_maybe_whitespace(),
                just(Token::SigilCloseSquareBracket),
            )
            .then(unit.clone().or_not())
            .map_with(|(((start, end), extras), unit), e| {
                ast::Expression::Range(ast::RangeExpression {
                    span: e.span(),
                    extras,
                    start: Box::new(start),
                    end: Box::new(end),
                    unit,
                })
            })
            .labelled("range")
            .boxed();

        let list = list_item
            .clone()
            .separated_by(just(Token::SigilComma).then_maybe_whitespace())
            .allow_trailing()
            .collect::<Vec<_>>()
            .with_extras()
            .delimited_by(
                just(Token::SigilOpenSquareBracket).then_maybe_whitespace(),
                just(Token::SigilCloseSquareBracket),
            )
            .then(unit.clone().or_not())
            .map_with(|((items, extras), unit), e| {
                ast::Expression::List(ast::ListExpression {
                    span: e.span(),
                    extras,
                    items,
                    unit,
                })
            })
            .labelled("list")
            .boxed();

        let body_expression = body
            .clone()
            .map(ast::Expression::Body)
            .labelled("body expression")
            .boxed();

        if_inner.define(
            just(Token::KeywordIf)
                .map_with(|_, e| e.span())
                .then_whitespace()
                .then(expression_parser.clone())
                .then_maybe_whitespace()
                .then(body.clone())
                .then_maybe_whitespace()
                .then(
                    just(Token::KeywordElse)
                        .map_with(|_, e| e.span())
                        .then_maybe_whitespace()
                        .then(if_inner.clone())
                        .map(|(span, inner)| (span, Box::new(inner)))
                        .or_not(),
                )
                .then(
                    just(Token::KeywordElse)
                        .map_with(|_, e| e.span())
                        .then_maybe_whitespace()
                        .then(body.clone())
                        .or_not(),
                )
                .with_extras()
                .map_with(
                    |(((((if_span, condition), body), next_if), else_body), extras), e| {
                        let (next_if_span, next_if) = next_if
                            .map(|(span, if_expr)| (Some(span), Some(if_expr)))
                            .unwrap_or((None, None));
                        let (else_span, else_body) = else_body
                            .map(|(span, body)| (Some(span), Some(body)))
                            .unwrap_or((None, None));
                        ast::If {
                            span: e.span(),
                            if_span,
                            extras,
                            condition: Box::new(condition),
                            body,
                            next_if_span,
                            next_if,
                            else_span,
                            else_body,
                        }
                    },
                )
                .boxed(),
        );
        let if_expression = if_inner
            .map(ast::Expression::If)
            .labelled("if expression")
            .boxed();

        let symbol_path_expr = symbol_path.clone().map(ast::Expression::SymbolPath).boxed();

        let call = call_inner
            .clone()
            .map(ast::Expression::Call)
            .labelled("method call");

        let bracket_based = bracketed.or(tuple).recover_with(via_parser(
            tuple_recovery
                .clone()
                .map_with(|_, e| ast::Expression::Error(e.span())),
        ));

        let base = literal
            .or(string_format)
            .or(if_expression)
            .or(call)
            .or(marker)
            .or(bracket_based)
            .or(range)
            .or(list)
            .or(body_expression)
            .or(symbol_path_expr)
            .boxed();

        let access_attribute = just(Token::SigilHash)
            .ignore_then(identifier_parser.clone())
            .map(ast::ElementInner::Attribute)
            .labelled("attribute access")
            .boxed();

        let access_tuple = just(Token::SigilDot)
            .ignore_then(identifier_parser.clone())
            .map(ast::ElementInner::Tuple)
            .labelled("tuple access")
            .boxed();

        let access_method = just(Token::SigilDot)
            .ignore_then(call_inner)
            .map(|call| ast::ElementInner::Method(Box::new(call)))
            .labelled("method call")
            .boxed();

        let list_access = expression_parser
            .clone()
            .delimited_by(
                just(Token::SigilOpenSquareBracket),
                just(Token::SigilCloseSquareBracket),
            )
            .map(Box::new)
            .map(ast::ElementInner::ListElement)
            .labelled("list access")
            .boxed();

        let access_item = access_attribute
            .or(access_method)
            .or(access_tuple)
            .or(list_access)
            .with_extras()
            .map_with(|(inner, extras), e| ast::Element {
                span: e.span(),
                inner,
                extras,
            })
            .repeated()
            .at_least(1)
            .collect::<Vec<ast::Element>>();

        let element_access = base
            .clone()
            .foldl_with(access_item.repeated(), |value, element_chain, e| {
                ast::Expression::ElementAccess(ast::ElementAccess {
                    span: e.span(),
                    expr: value.into(),
                    element_chain,
                })
            })
            .labelled("element access")
            .boxed();

        let unary_expression = unary_operator_parser
            .then_maybe_whitespace()
            .then(element_access.clone())
            .with_extras()
            .map_with(|((op, rhs), extras), e| {
                ast::Expression::UnaryOperation(ast::UnaryOperation {
                    span: e.span(),
                    extras,
                    op,
                    rhs: rhs.into(),
                })
            })
            .boxed();

        let binary_param = element_access.or(unary_expression.clone());

        let near = binop(binary_param, &[Token::OperatorNear]);
        let xor = binop(near, &[Token::OperatorPowerXor, Token::OperatorXor]);
        let union_intersect = binop(xor, &[Token::OperatorUnion, Token::OperatorIntersect]);
        let mul_div = binop(
            union_intersect,
            &[Token::OperatorMultiply, Token::OperatorDivide],
        );
        let add_sub = binop(mul_div, &[Token::OperatorAdd, Token::OperatorSubtract]);
        let less_greater_eq = binop(
            add_sub,
            &[Token::OperatorLessEqual, Token::OperatorGreaterEqual],
        );
        let less_greater = binop(
            less_greater_eq,
            &[Token::OperatorLessThan, Token::OperatorGreaterThan],
        );
        let eq_neq = binop(
            less_greater,
            &[Token::OperatorEqual, Token::OperatorNotEqual],
        );
        let or_and = binop(eq_neq, &[Token::OperatorOr, Token::OperatorAnd]);

        or_and.labelled("expression").boxed()
    });

    statement_list_parser
        .then_ignore(end())
        .map_with(move |statements, ex| ast::Source {
            span: ex.span(),
            statements,
        })
}

impl crate::Parse for ast::Literal {
    fn parse(context: &ParseContext) -> Result<Self, Vec<ParseError>> {
        fn literal<'tokens>()
        -> impl Parser<'tokens, ParserInput<'tokens, 'tokens>, ast::Literal, Extra<'tokens>>
        {
            ast::Literal::parser()
        }

        let tokens = crate::lex::lex(context.source.code()).collect::<Vec<_>>();
        literal()
            .parse(crate::parse::input(&tokens))
            .into_result()
            .map_err(|errors| {
                errors
                    .into_iter()
                    .map(|err| ParseError::new(err, context))
                    .collect::<Vec<_>>()
            })
    }
}
