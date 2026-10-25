// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Diagnostics, IssueList, Source};
use mucad_lang_parse::{Ast, ParseIssue, ast, parse};

mod expression;
mod extras;
mod literal;
mod node;
mod statement;
mod ty;

pub(crate) use crate::node::{BreakMode, Node};

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub max_width: usize,
    pub indent_width: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            max_width: 60,
            indent_width: 4,
        }
    }
}

pub(crate) trait Format {
    fn format(&self, f: &FormatConfig) -> Node;
}

impl<T> Format for T
where
    T: Into<Node> + Clone,
{
    fn format(&self, _f: &FormatConfig) -> Node {
        self.clone().into()
    }
}

// Blanket impl for Option
impl<T: Format> Format for Option<T> {
    fn format(&self, f: &FormatConfig) -> Node {
        match self {
            Some(inner) => inner.format(f),
            None => Node::Nil, // Or whatever your "null" node is
        }
    }
}

impl Format for ast::Identifier {
    fn format(&self, _: &FormatConfig) -> Node {
        self.name.clone().into()
    }
}

impl Format for ast::Comment {
    fn format(&self, _: &FormatConfig) -> Node {
        match &self.inner {
            ast::CommentInner::SingleLine(line) => {
                node!(Node::Softline Node::SingleLineComment(line.into()))
            }
            ast::CommentInner::MultiLine(line) => {
                node!(Node::Softline "/* " Node::from(line.clone()) " */" Node::Softline)
            }
        }
    }
}

impl Format for ast::DocBlock {
    fn format(&self, _: &FormatConfig) -> Node {
        Node::vlist(
            self.lines.iter().cloned().map(|line| node!(line)),
            Node::Nil,
            0,
        )
    }
}

impl Format for ast::Source {
    fn format(&self, f: &FormatConfig) -> Node {
        self.statements.format(f)
    }
}

impl Format for Ast {
    fn format(&self, f: &FormatConfig) -> Node {
        self.tree().format(f)
    }
}

/// node! macro for syntactic suger.
#[macro_export]
macro_rules! node {
    // Single element: node!(x)
    ($node:expr) => {
        $crate::Node::from($node)
    };
    // Multiple elements: node!(begin, body, end)
    ($($node:expr)*) => {
        $crate::Node::from(vec![
            $( $crate::Node::from($node) ),*
        ])
    };
    // Multiple formatted elements with extras: node!(f, extras => begin, body, end)
    ($f:ident, $extras:expr => $($node:expr)*) => {
        $crate::extras::with_extras(
            &$extras,
            $f,
            $crate::Node::from(vec![
                $( $node.format($f) ),*
            ])

        )
    };
    // Multiple formatted elements: node!(f => begin, body, end)
    ($f:ident => $($node:expr)*) => {
        $crate::Node::from(vec![
            $( $node.format($f) ),*
        ])
    };
}

/// Format µcad a parsed abstract syntax tree into a string
pub fn format_ast(ast: &Ast, config: &FormatConfig) -> String {
    ast.format(config).to_string()
}

/// A formatted source and AST.
pub struct Formatted {
    pub source: Source,
    pub ast: Ast,
    pub formatted: bool,
    pub issues: IssueList<ParseIssue>,
}

/// Format an AST into a new AST and a new Source
pub fn format(
    source: impl Into<Source>,
    config: &FormatConfig,
) -> Result<Formatted, IssueList<ParseIssue>> {
    let original_source = source.into();
    let (ast, _) = parse(&original_source)?;
    let code = format_ast(&ast, config);
    let source = Source::new(original_source.location.clone(), code);
    parse(&source).map(|(ast, issues)| Formatted {
        formatted: source != original_source,
        ast,
        source,
        issues,
    })
}

/// High-level API to format a &str containing µcad source code.
pub fn format_code(code: &str, config: &FormatConfig) -> Result<String, Diagnostics> {
    let source = Source::from(code);
    let ast = parse(&source)?.0;
    Ok(format_ast(&ast, config))
}
