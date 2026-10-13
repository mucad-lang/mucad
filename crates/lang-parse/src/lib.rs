// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Syntax definitions and parser for µcad source code.
//!
//! This module includes the components to parse µcad source code into a stream of tokens or abstract syntax tree.
//!
//! - Transform source code into a stream of tokens with [`lex`]
//! - Create an abstract syntax tree from the list of tokens with [`parse`]

/// Abstract syntax tree for µcad files
pub mod ast;

/// Tokens
pub mod token;

#[cfg(feature = "parser")]
use microcad_lang_base::{IssueList, ToHash};
#[cfg(feature = "parser")]
pub use parse::{ParseContext, ParseError, ParseInfo, ParseIssue, ParseWarning, parsers};

/// Contains the parser.
#[cfg(feature = "parser")]
mod parse;

/// Contains the lexer (aka tokenizer).
mod lex;

use microcad_lang_base::{HashId, IssueResult};
use microcad_macros::Artifact;

pub use lex::lex;

use serde::Serialize;

/// A parsed abstract syntax tree with hashes.
#[derive(Debug, Artifact, Serialize)]
pub struct Ast {
    input_hash: HashId,
    output_hash: HashId,
    tree: ast::Source,
}

impl Ast {
    /// Return the tree for [`Ast`].
    pub fn tree(&self) -> &ast::Source {
        &self.tree
    }

    /// Input hash.
    pub fn input_hash(&self) -> HashId {
        self.input_hash
    }

    /// Output hash.
    pub fn output_hash(&self) -> HashId {
        self.output_hash
    }
}

impl From<Ast> for ast::Source {
    fn from(ast: Ast) -> Self {
        ast.tree
    }
}

/// Parse trait.
#[cfg(feature = "parser")]
pub trait Parse: Sized {
    /// Parse from a context.
    ///
    /// The context also contains the source string.
    fn parse(context: &ParseContext) -> Result<Self, Vec<ParseError>>;
}

#[cfg(feature = "parser")]
impl Parse for Ast {
    fn parse(context: &ParseContext) -> Result<Self, Vec<ParseError>> {
        let tokens = lex(context.source.code()).collect::<Vec<_>>();
        let tree = parse::parse(&tokens, context)?;

        Ok(Self {
            input_hash: context.source.hash_id(),
            output_hash: tree.to_hash(),
            tree,
        })
    }
}

/// Parse a source into an abstract syntax tree.
#[cfg(feature = "parser")]
pub fn parse<'source>(context: impl Into<ParseContext<'source>>) -> IssueResult<Ast, ParseIssue> {
    let context = context.into();
    match Ast::parse(&context) {
        Ok(ast) => Ok((ast, IssueList::default())), // FIXME: Right now, the parser can only return errors and no warnings
        Err(errors) => Err(errors.into()),
    }
}
