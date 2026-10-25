// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{BreakMode, Format, FormatConfig, Node, extras::leading_extras_without_newline, node};

use mucad_lang_parse::{ast, token::escape_char};

impl Format for ast::BinaryOperator {
    fn format(&self, _: &FormatConfig) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::UnaryOperator {
    fn format(&self, _: &FormatConfig) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::Body {
    fn format(&self, f: &FormatConfig) -> Node {
        let body = &self.statements;

        match (body.statements.is_empty(), &body.tail) {
            (true, Some(tail)) => {
                let tail_node = node!(f, body.extras => tail.format(f));
                if tail_node.contains_hardline() {
                    node!(
                        "{" Node::indent(f.indent_width, tail_node)
                        "}"
                    )
                } else {
                    node!("{ " tail_node " }")
                }
            }
            (true, None) => {
                node!("{" Node::indent(f.indent_width, node!(f, body.extras => Node::Nil)) "}")
            }
            _ => {
                let leading = body.extras.leading.format(f);
                let node = Node::indent(f.indent_width, body.format(f));
                node!(
                "{" if leading.starts_with_hardline() { Node::Nil } else { Node::Hardline }
                    node.clone()
                    if node.ends_with_hardline() { Node::Nil } else { Node::Hardline }
                "}")
            }
        }
    }
}

impl Format for ast::Expression {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            ast::Expression::Literal(literal) => literal.format(f),
            ast::Expression::Bracketed(bracket, _) => node!('(' bracket.format(f) ')'),
            ast::Expression::Tuple(tuple_expression) => tuple_expression.format(f),
            ast::Expression::Range(range_expr) => range_expr.format(f),
            ast::Expression::List(list_expr) => list_expr.format(f),
            ast::Expression::String(format_string) => format_string.format(f),
            ast::Expression::SymbolPath(qualified_name) => qualified_name.format(f),
            ast::Expression::Marker(identifier) => format!("@{}", identifier.name).into(),
            ast::Expression::BinaryOperation(binary_operation) => binary_operation.format(f),
            ast::Expression::UnaryOperation(unary_operation) => unary_operation.format(f),
            ast::Expression::Body(body) => body.format(f),
            ast::Expression::Call(call) => call.format(f),
            ast::Expression::ElementAccess(element_access) => element_access.format(f),
            ast::Expression::If(i) => i.format(f),
            ast::Expression::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::StringPart {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            ast::StringPart::Char(string_character) => string_character.format(f),
            ast::StringPart::Content(string_literal) =>
            // Simply clone the content of the string literal, because we do not want to have quotes '"'
            {
                string_literal.content.clone().into()
            }
            ast::StringPart::Expression(string_expression) => string_expression.format(f),
        }
    }
}

impl Format for ast::StringCharacter {
    fn format<'a>(&self, _: &FormatConfig) -> Node {
        // An escaped character, e.g. `\/`, which is no escape sequence by `escape_char`.
        match escape_char(self.character) {
            Some(escaped) => escaped.into(),
            None => format!("\\{}", self.character).into(),
        }
    }
}

impl Format for ast::StringExpression {
    fn format<'a>(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            '{' self.expr self.specification '}'
        )
    }
}

impl Format for ast::StringFormatSpecification {
    fn format<'a>(&self, _f: &FormatConfig) -> Node {
        match (&self.width, &self.precision) {
            (Some(Ok(width)), Some(Ok(precision))) => format!(":0{width}.{precision}").into(),
            (None, Some(Ok(precision))) => format!(":.{precision}").into(),
            (Some(Ok(width)), None) => format!(":0{width}").into(),
            _ => Node::Nil,
        }
    }
}

impl Format for ast::FormatString {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            '"'
            self.parts
                .iter()
                .map(|part| part.format(f))
                .collect::<Vec<_>>()
            '"'
        )
    }
}

impl Format for ast::TupleItem {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, leading_extras_without_newline(&self.extras) =>
            match &self.id {
                Some(name) => node!(f => name " = " self.expr),
                None => node!(f => self.expr)
            }
        )
    }
}

impl Format for ast::TupleExpression {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.values.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 4, f);
        node!(f, leading_extras_without_newline(&self.extras) =>
            '(' Node::list(nodes, ',', break_mode) ')'
        )
    }
}

impl Format for ast::ListItem {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.expr)
    }
}

impl Format for ast::RangeExpression {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            '[' self.start ".." self.end ']' self.unit
        )
    }
}

impl Format for ast::ListExpression {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.items.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 0, f);
        node!(f, self.extras =>
            '[' Node::list(nodes, ',', break_mode) ']' self.unit
        )
    }
}

impl Format for ast::SymbolPath {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            if self.prefix.is_some() { "::" } else { "" }
            Node::hlist(self.parts.iter().map(|identifier| identifier.format(f)), "::")
        )
    }
}

impl Format for ast::BinaryOperation {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f => self.lhs Node::Softline self.op Node::Softline self.rhs)
    }
}

impl Format for ast::UnaryOperation {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f => self.op self.rhs)
    }
}

impl Format for ast::Argument {
    fn format(&self, f: &FormatConfig) -> Node {
        match self {
            ast::Argument::Unnamed(arg) => arg.format(f),
            ast::Argument::Named(arg) => arg.format(f),
        }
    }
}

impl Format for ast::UnnamedArgument {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.expr)
    }
}

impl Format for ast::NamedArgument {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.id " = " self.expr)
    }
}

impl Format for ast::ArgumentList {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.arguments.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 4, f);

        node!('(' node!(f, leading_extras_without_newline(&self.extras) => Node::list(nodes, ',', break_mode)) ')')
    }
}

impl Format for ast::Call {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras => self.path self.arguments)
    }
}

impl Format for ast::ElementInner {
    fn format(&self, f: &FormatConfig) -> Node {
        use ast::ElementInner::*;
        match &self {
            Attribute(identifier) => node!(f => '#' identifier),
            Tuple(identifier) => node!(f => '.' identifier),
            Method(call) => node!(f => '.' call),
            ListElement(expression) => node!(f => '[' expression ']'),
        }
    }
}

impl Format for ast::Element {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, leading_extras_without_newline(&self.extras) => self.inner)
    }
}

impl Format for ast::ElementAccess {
    fn format(&self, f: &FormatConfig) -> Node {
        // If this is true, we place an indent on the next line
        let mut indent = match &self.expr.as_ref() {
            ast::Expression::Literal(_) => false,
            ast::Expression::Bracketed(_, _) => true,
            ast::Expression::Tuple(_) => true,
            ast::Expression::Range(_) => false,
            ast::Expression::List(_) => true,
            ast::Expression::String(_) => true,
            ast::Expression::SymbolPath(_) => true,
            ast::Expression::Marker(_) => true,
            ast::Expression::BinaryOperation(_) => true,
            ast::Expression::UnaryOperation(_) => true,
            ast::Expression::Body(_) => false,
            ast::Expression::Call(_) => true,
            ast::Expression::ElementAccess(_) => false,
            ast::Expression::If(_) => false,
            ast::Expression::Error(_) => false,
        };

        let nodes: Vec<Node> = self
            .element_chain
            .iter()
            .map(|element| node!(f => element))
            .collect();

        // Indent if the first node already starts with a hardline
        indent |= nodes
            .iter()
            .take(1)
            .any(|node| !node.starts_with_hardline());

        let element_chain_node = match BreakMode::from_layout(&nodes, 3, f) {
            BreakMode::NoBreak if !indent => Node::hlist(nodes, Node::Nil),
            BreakMode::NoBreak => Node::indent(f.indent_width, nodes),
            BreakMode::WithIndent(indent_width) => {
                Node::indent(if indent { indent_width } else { 0 }, nodes)
            }
        };

        node!(f => self.expr element_chain_node)
    }
}

impl Format for ast::If {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            "if " self.condition ' ' self.body
            match &self.else_body {
                Some(else_body) => node!(f => Node::Softline "else " else_body),
                None => match &self.next_if {
                    Some(_) => Node::Nil,
                    None => Node::Hardline
                }
            }
            match &self.next_if {
                Some(next_if) => node!(f => Node::Softline "else " next_if),
                None => Node::Nil
            }
        )
    }
}
