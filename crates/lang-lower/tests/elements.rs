// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{self as base};
use mucad_lang_lower::{Desugar, LowerContext, ir};
use mucad_lang_parse::ast;

mod common;

#[test]
fn outer_doc_block() {
    // An AST still has the /// prefix
    // The IR has stripped it of
    let doc_block = ast::DocBlock {
        span: 0..16,
        lines: vec!["/// A".to_string(), "/// B".to_string(), "///C".to_string()],
    };

    let source = base::Source::new(base::SourceKind::Str, String::new());
    let mut lower_context = LowerContext::from(&source);
    let doc_block = ir::DocBlock::desugar(&doc_block, &mut lower_context).unwrap();

    let lines = doc_block.content.lines().collect::<Vec<_>>();

    assert_eq!(vec!["A", "B", "C"], lines);
}

#[test]
fn inner_doc_block() {
    // An AST still has the //! prefix
    // The IR has stripped it of
    fn inner_doc_comment(s: &str) -> (ast::Statement, ast::TrailingExtras) {
        (
            ast::Statement::InnerDocComment(ast::InnerDocComment {
                span: 0..0,
                line: s.to_string(),
            }),
            ast::TrailingExtras::default(),
        )
    }

    let statements = ast::StatementList {
        span: 0..16,
        extras: ast::ItemExtras::default(),
        statements: vec![
            inner_doc_comment("//! A"),
            inner_doc_comment("//! B"),
            inner_doc_comment("//!C"),
        ],
        tail: None,
    };

    let source = base::Source::from("");
    let mut lower_context = LowerContext::from(&source);
    let doc_block = ir::DocBlock::desugar(&statements, &mut lower_context).unwrap();

    let lines = doc_block.content.lines().collect::<Vec<_>>();

    assert_eq!(vec!["A", "B", "C"], lines);
}
