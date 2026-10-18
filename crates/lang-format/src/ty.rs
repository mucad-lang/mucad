// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{BreakMode, Format, FormatConfig, Node, node};

use mucad_lang_parse::ast;

impl Format for ast::Unit {
    fn format(&self, _: &FormatConfig) -> Node {
        self.name.clone().into()
    }
}

impl Format for ast::Type {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            ast::Type::Single(single_type) => single_type.format(f),
            ast::Type::List(list_type) => list_type.format(f),
            ast::Type::Tuple(tuple_type) => tuple_type.format(f),
        }
    }
}

impl Format for ast::SingleType {
    fn format(&self, _: &FormatConfig) -> Node {
        self.name.clone().into()
    }
}

impl Format for ast::ListType {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f => '[' self.inner ']')
    }
}

impl Format for ast::TupleType {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self
            .inner
            .iter()
            .map(|item| match &item.0 {
                Some(name) => node!(f => name " = " item.1),
                None => item.1.format(f),
            })
            .collect();
        let break_mode = BreakMode::from_layout(&nodes, 4, f);

        node!('(' Node::list(nodes, ',', break_mode) ')')
    }
}
