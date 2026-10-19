// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod expression;
mod statement;

use mucad_driver::prelude as mu;

use mu::ast;

use lsp::SemanticTokenModifier as TokenModifier;
use lsp::SemanticTokenType as TokenType;
use tower_lsp::lsp_types as lsp;
use tower_lsp::lsp_types::SemanticToken;

use crate::to_lsp::position;

/// The semantic tokens of a parsed `source`, encoded relative to each other as LSP expects.
pub(crate) fn semantic_tokens(source: &mu::Source, tree: &ast::Source) -> Vec<SemanticToken> {
    let mut ctx = TokenContext::default();
    tree.semantic_tokens(&mut ctx);
    ctx.encode(source.code())
}

/// Collects the tokens of an AST in any order.
#[derive(Default)]
pub(crate) struct TokenContext {
    tokens: Vec<(mu::base::Span, TokenType, u32)>,
}

/// Macro for token implementation
#[macro_export]
macro_rules! impl_tokens {
    // 3. Field List Arm: Triggers for a list of comma-separated structural fields
    ($t:ty => $($field:ident),+ $(,)?) => {
        impl SemanticTokens for $t {
            fn semantic_tokens(&self, ctx: &mut TokenContext) {
                $(
                    self.$field.semantic_tokens(ctx);
                )+
            }
        }
    };

    // 1. Single Token Arm: Triggers when the right side is a type path (e.g., TokenType::COMMENT)
    ($t:ty => $token_type:path) => {
        impl SemanticTokens for $t {
            fn semantic_tokens(&self, ctx: &mut TokenContext) {
                ctx.push_token(&self.span, $token_type, &[]);
            }
        }
    };

    // 4. Custom Block Arm: Fallback for matches or custom logic
    ($t:ty => |$self:ident, $ctx:ident| $body:block) => {
        impl SemanticTokens for $t {
            fn semantic_tokens(&self, $ctx: &mut TokenContext) {
                let $self = self;
                $body
            }
        }
    };
}

impl TokenContext {
    fn push_token(
        &mut self,
        span: &mu::base::Span,
        token_type: TokenType,
        modifiers: &[lsp::SemanticTokenModifier],
    ) {
        if !span.is_empty() {
            self.tokens.push((
                span.clone(),
                token_type,
                semantic_token_modifier_bitset(modifiers),
            ));
        }
    }

    /// Encode the tokens sorted by position, a token over several lines is split into lines.
    fn encode(mut self, code: &str) -> Vec<SemanticToken> {
        self.tokens.sort_by_key(|(span, ..)| span.start);

        let mut last = lsp::Position::default();
        let mut encoded = Vec::new();
        for (span, token_type, modifiers) in self.tokens {
            let mut line_start = span.start;
            for line in code[span.clone()].split_inclusive('\n') {
                let text = line.trim_end_matches(['\n', '\r']);
                let start = position(code, line_start);
                line_start += line.len();
                if text.is_empty() {
                    continue;
                }

                let delta_line = start.line - last.line;
                encoded.push(SemanticToken {
                    delta_line,
                    delta_start: match delta_line {
                        0 => start.character - last.character,
                        _ => start.character,
                    },
                    length: text.encode_utf16().count() as u32,
                    token_type: semantic_token_type_index(&token_type),
                    token_modifiers_bitset: modifiers,
                });
                last = start;
            }
        }
        encoded
    }
}

pub trait SemanticTokens {
    fn semantic_tokens(&self, ctx: &mut TokenContext);
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

impl_tokens!(ast::Source => statements);

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
