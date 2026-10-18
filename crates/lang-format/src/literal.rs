// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Format, FormatConfig, Node, node};

use mucad_lang_parse::ast;

impl Format for ast::Literal {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.literal)
    }
}

impl Format for ast::LiteralKind {
    fn format(&self, f: &FormatConfig) -> Node {
        match self {
            ast::LiteralKind::String(lit) => lit.format(f),
            ast::LiteralKind::Bool(lit) => lit.format(f),
            ast::LiteralKind::Integer(lit) => lit.format(f),
            ast::LiteralKind::Float(lit) => lit.format(f),
            ast::LiteralKind::Quantity(lit) => lit.format(f),
            ast::LiteralKind::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::StringLiteral {
    fn format(&self, _: &FormatConfig) -> Node {
        format!("\"{}\"", self.content).into()
    }
}

impl Format for ast::BoolLiteral {
    fn format(&self, _: &FormatConfig) -> Node {
        if self.value { "true" } else { "false" }.into()
    }
}

impl Format for ast::IntegerLiteral {
    fn format(&self, _: &FormatConfig) -> Node {
        self.value.clone().into()
    }
}

impl Format for ast::FloatLiteral {
    fn format(&self, _: &FormatConfig) -> Node {
        self.value.clone().into()
    }
}

impl Format for ast::QuantityLiteral {
    fn format(&self, _: &FormatConfig) -> Node {
        format!("{}{}", self.value, self.unit.name).into()
    }
}
