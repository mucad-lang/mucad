// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod expression;
mod statement;

use mucad_driver::prelude as mu;
use mucad_driver::prelude::Diagnostics;
use mucad_driver::prelude::base::LineCol;

use mu::ast;
use mu::traits::*;

use lsp::SemanticTokenModifier as TokenModifier;
use lsp::SemanticTokenType as TokenType;
use tower_lsp::lsp_types as lsp;
use tower_lsp::lsp_types::SemanticToken;

use crate::to_lsp::ToLsp;

pub(crate) struct TokenContext<'ast> {
    source: &'ast ast::Source,
    line_index: mu::base::LineIndex,
    last: mu::base::LineCol,
    tokens: Vec<SemanticToken>,
    diag: mu::Diagnostics,
}

/// Macro for token implementation
#[macro_export]
macro_rules! impl_tokens {
    // 3. Field List Arm: Triggers for a list of comma-separated structural fields
    ($t:ty => $($field:ident),+ $(,)?) => {
        impl<'ast> SemanticTokens<'ast> for $t {
            fn semantic_tokens(&self, ctx: &'ast mut TokenContext) {
                $(
                    self.$field.semantic_tokens(ctx);
                )+
            }
        }
    };

    // 1. Single Token Arm: Triggers when the right side is a type path (e.g., TokenType::COMMENT)
    ($t:ty => $token_type:path) => {
        impl<'ast> SemanticTokens<'ast> for $t {
            fn semantic_tokens(&self, ctx: &'ast mut TokenContext) {
                ctx.push_token(&self.span, $token_type, &[]);
            }
        }
    };

    // 4. Custom Block Arm: Fallback for matches or custom logic
    ($t:ty => |$self:ident, $ctx:ident| $body:block) => {
        impl<'ast> SemanticTokens<'ast> for $t {
            fn semantic_tokens(&self, $ctx: &'ast mut TokenContext) {
                let $self = self;
                $body
            }
        }
    };
}

impl<'ast> TokenContext<'ast> {
    pub fn new(source: &'ast ast::Source) -> Self {
        Self {
            source,
            line_index: mu::base::LineIndex::new(&source.code),
            last: LineCol::default(),
            tokens: Vec::default(),
            diag: Diagnostics::default(),
        }
    }

    pub fn tokens(&self) -> &Vec<SemanticToken> {
        &self.tokens
    }

    fn span_to_src_ref(&self, span: &mu::base::Span) -> mu::base::SrcRef {
        self.line_index
            .src_ref(&self.source.code, span, self.source.code.computed_hash())
    }

    fn error(&mut self, span: &mu::base::Span, err: miette::ErrReport) {
        self.diag
            .error(&self.span_to_src_ref(span), err)
            .expect("No error");
    }

    fn push_token(
        &mut self,
        span: &mu::base::Span,
        token_type: TokenType,
        modifiers: &[lsp::SemanticTokenModifier],
    ) {
        let src_ref = self.span_to_src_ref(span);
        assert!(src_ref.is_some());

        let length = src_ref.len() as u32;
        let pos = src_ref.to_lsp().expect("A src_ref with a hash");
        let line = pos.start.line;
        let char = pos.start.character;
        let delta_start = if line == self.last.line {
            char - self.last.col
        } else {
            char
        };

        match line.checked_sub(self.last.line) {
            Some(delta_line) => {
                self.last = mu::base::LineCol { line, col: char };

                self.tokens.push(lsp::SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: semantic_token_type_index(&token_type),
                    token_modifiers_bitset: semantic_token_modifier_bitset(modifiers),
                });
            }
            None => {
                self.error(span, miette::miette!("Line overflow"));
            }
        }
    }
}

pub trait SemanticTokens<'ast> {
    fn semantic_tokens(&'ast self, ctx: &'ast mut TokenContext);
}

impl_tokens!(ast::Comment => TokenType::COMMENT);
impl_tokens!(ast::Unit => TokenType::KEYWORD); // TODO: A custom "unit" token type is preferred

impl_tokens!(ast::ListType => inner);
impl_tokens!(ast::TupleType => |self_, ctx| {
    self_.inner.iter().for_each(|(id, ty)| {
        ty.semantic_tokens(ctx);
         if let Some(id) = id.as_ref() { ctx.push_token(&id.span, TokenType::PROPERTY, &[]) };
    });
});

impl_tokens!(ast::SingleType => TokenType::TYPE);
impl_tokens!(ast::Type => |self_, ctx| {
    match self_ {
        ast::Type::List(a) => a.semantic_tokens(ctx),
        ast::Type::Single(s) => s.semantic_tokens(ctx),
        ast::Type::Tuple(t) => t.semantic_tokens(ctx),
    }
});

impl_tokens!(ast::ItemExtra => |self_, ctx| {
    if let ast::ItemExtra::Comment(comment) = self_ {
        comment.semantic_tokens(ctx);
    }
});

impl_tokens!(ast::TrailingExtras => |self_, ctx| {
    self_.0.iter().for_each(|i| i.semantic_tokens(ctx));
});

impl_tokens!(ast::LeadingExtras => |self_, ctx| {
    self_.0.iter().for_each(|i| i.semantic_tokens(ctx));
});

impl_tokens!(ast::ItemExtras => leading, trailing);

impl_tokens!(ast::Program => statements);

const SEMANTIC_LENGTH: TokenType = TokenType::new("length");

pub const LEGEND_TYPES: &[TokenType] = &[
    TokenType::NAMESPACE,
    TokenType::TYPE,
    TokenType::CLASS,
    TokenType::ENUM,
    TokenType::INTERFACE,
    TokenType::STRUCT,
    TokenType::TYPE_PARAMETER,
    TokenType::PARAMETER,
    TokenType::VARIABLE,
    TokenType::PROPERTY,
    TokenType::ENUM_MEMBER,
    TokenType::EVENT,
    TokenType::FUNCTION,
    TokenType::METHOD,
    TokenType::MACRO,
    TokenType::KEYWORD,
    TokenType::MODIFIER,
    TokenType::COMMENT,
    TokenType::STRING,
    TokenType::NUMBER,
    SEMANTIC_LENGTH,
    TokenType::REGEXP,
    TokenType::OPERATOR,
    TokenType::DECORATOR,
];

pub const LEGEND_MODIFIERS: &[TokenModifier] = &[
    TokenModifier::DECLARATION,
    TokenModifier::DEFINITION,
    TokenModifier::READONLY,
    TokenModifier::STATIC,
    TokenModifier::DEPRECATED,
    TokenModifier::ABSTRACT,
    TokenModifier::ASYNC,
    TokenModifier::MODIFICATION,
    TokenModifier::DOCUMENTATION,
    TokenModifier::DEFAULT_LIBRARY,
];

fn semantic_token_type_index(token_type: &TokenType) -> u32 {
    LEGEND_TYPES
        .iter()
        .position(|t| t == token_type)
        .expect("Token type not found in legend") as u32
}

fn semantic_token_modifier_index(token_modifier: &TokenModifier) -> u32 {
    1 << LEGEND_MODIFIERS
        .iter()
        .position(|t| t == token_modifier)
        .expect("Token modifier not found in legend") as u32
}

fn semantic_token_modifier_bitset(token_modifiers: &[TokenModifier]) -> u32 {
    token_modifiers
        .iter()
        .map(semantic_token_modifier_index)
        .reduce(|a, b| a | b)
        .unwrap_or(0)
}
