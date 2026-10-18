// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{BreakMode, Format, FormatConfig, Node, extras::leading_extras_without_newline, node};

use mucad_lang_parse::ast;

impl Format for Option<ast::def::Visibility> {
    fn format(&self, _: &FormatConfig) -> Node {
        match &self {
            Some(ast::def::Visibility::Public) => "pub ".into(),
            _ => Node::Nil,
        }
    }
}

impl Format for ast::Parameter {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, leading_extras_without_newline(&self.extras) =>
            match (&self.ty, &self.default) {
                (None, None) => self.id.format(f),
                (None, Some(def)) => node!(f => self.id " = " def),
                (Some(ty), None) => node!(f => self.id ": " ty),
                (Some(ty), Some(def)) => node!(f => self.id ": " ty " = " def),
            }
        )
    }
}

impl Format for ast::ParameterList {
    fn format(&self, f: &FormatConfig) -> Node {
        let nodes: Vec<Node> = self.parameters.iter().map(|item| item.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 4, f);

        node!(f =>
            '(' node!(f, leading_extras_without_newline(&self.extras) => Node::list(nodes, ',', break_mode)) ')'
        )
    }
}

impl Format for ast::def::WorkbenchKind {
    fn format(&self, _: &FormatConfig) -> Node {
        self.to_string().into()
    }
}

impl Format for ast::def::Workbench {
    fn format(&self, f: &FormatConfig) -> Node {
        let vis = self.vis.as_ref().map(|vis| vis.value.clone());
        node!(f, self.extras =>
            self.doc
            self.attr
            vis self.kind ' ' self.id
            self.parameters ' '
            self.body
        )
    }
}

impl Format for ast::def::InlineModule {
    fn format(&self, f: &FormatConfig) -> Node {
        let vis = self.vis.as_ref().map(|vis| vis.value.clone());
        node!(f, self.extras =>
            self.doc
            self.attr
            vis "mod " self.id ' '
            self.body
        )
    }
}

impl Format for ast::def::FileModule {
    fn format(&self, f: &FormatConfig) -> Node {
        let vis = self.vis.as_ref().map(|vis| vis.value.clone());
        node!(f, self.extras =>
            self.doc
            self.attr
            vis "mod " self.id
        )
    }
}

impl Format for ast::def::Function {
    fn format(&self, f: &FormatConfig) -> Node {
        let return_type = match &self.return_type {
            Some(ty) => node!(f => "-> " ty " "),
            None => Node::Nil,
        };
        let vis = self.vis.as_ref().map(|vis| vis.value.clone());

        node!(f, self.extras =>
            self.doc
            self.attr
            vis "fn " self.id self.parameters " " return_type
            self.body
        )
    }
}

impl Format for ast::def::UseStatementPart {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            Self::Identifier(identifier) => identifier.format(f),
            Self::Glob(_) => '*'.into(),
            Self::Error(_) => Node::Nil,
        }
    }
}

impl Format for ast::def::UseName {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            if self.prefix.is_some() { "::" } else { "" }
            Node::hlist(self.parts.iter().map(|part| part.format(f)), "::")
        )
    }
}

impl Format for ast::def::Use {
    fn format(&self, f: &FormatConfig) -> Node {
        let vis = self.vis.as_ref().map(|vis| vis.value.clone());

        node!(f, self.extras =>
            self.attr
            vis "use " self.name
            self.use_as.as_ref().map(|ident| node!(f => " as " ident))
        )
    }
}

impl Format for ast::def::Constant {
    fn format(&self, f: &FormatConfig) -> Node {
        let vis = self.vis.as_ref().map(|vis| vis.value.clone());
        node!(f, self.extras =>
            self.doc
            self.attr
            vis "const " self.id " = " self.expr
        )
    }
}

impl Format for ast::Init {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            self.doc
            self.attr
            "init" self.parameters Node::Softline self.body
        )
    }
}

impl Format for ast::Return {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            "return"
            match &self.expr {
                Some(value) => node!(f => Node::Softline value),
                None => Node::Nil
            }
        )
    }
}

impl Format for ast::LocalAssignment {
    fn format(&self, f: &FormatConfig) -> Node {
        let assignment = node!(f =>
            self.id
            match &self.ty {
                Some(ty) => node!(f => ':' Node::Softline ty),
                None => Node::Nil,
            }
            Node::Softline '=' Node::Softline
        );

        node!(f, self.extras =>
            self.attr
            assignment Node::AdditionalIndent(assignment.estimate_width()) self.expr
        )
    }
}

impl Format for ast::PropertyAssignment {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            self.doc
            self.attr
            "prop" Node::Softline self.id
            self.ty.as_ref().map(|ty| node!(f => ':' Node::Softline ty))
            " = " self.value
        )
    }
}

impl Format for ast::ExpressionStatement {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            self.attr
            self.expr
        )
    }
}

impl Format for ast::InnerDocComment {
    fn format(&self, _: &FormatConfig) -> Node {
        self.line.clone().into()
    }
}

impl Format for ast::AttributeCommand {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            ast::AttributeCommand::Ident(identifier) => identifier.format(f),
            ast::AttributeCommand::Assignment(local_assignment) => local_assignment.format(f),
            ast::AttributeCommand::Call(call) => call.format(f),
        }
    }
}

impl Format for ast::Attribute {
    fn format(&self, f: &FormatConfig) -> Node {
        let prefix = if self.is_inner { "#![" } else { "#[" };

        let nodes: Vec<Node> = self.commands.iter().map(|attr| attr.format(f)).collect();
        let break_mode = BreakMode::from_layout(&nodes, 0, f);

        node!(f, self.extras =>
            prefix Node::list(nodes, ',', break_mode) ']'
        )
    }
}

impl Format for Vec<ast::Attribute> {
    fn format(&self, f: &FormatConfig) -> Node {
        if self.is_empty() {
            Node::Nil
        } else {
            self.iter()
                .map(|attr| {
                    let node = attr.format(f);
                    if node.contains_hardline() {
                        node
                    } else {
                        node!(node Node::Hardline)
                    }
                })
                .collect::<Vec<_>>()
                .into()
        }
    }
}

impl Format for ast::Attributes {
    fn format(&self, f: &FormatConfig) -> Node {
        self.0.format(f)
    }
}

impl Format for ast::Statement {
    fn format(&self, f: &FormatConfig) -> Node {
        match &self {
            Self::Workbench(workbench_definition) => workbench_definition.format(f),
            Self::InlineModule(inline_module) => inline_module.format(f),
            Self::FileModule(file_module) => file_module.format(f),
            Self::Function(function_definition) => function_definition.format(f),
            Self::InnerDocComment(comment) => comment.format(f),
            Self::Use(use_statement) => use_statement.format(f),
            Self::Const(const_assignment) => const_assignment.format(f),
            Self::Init(init_definition) => init_definition.format(f),
            Self::Return(r) => r.format(f),
            Self::InnerAttribute(attribute) => attribute.format(f),
            Self::LocalAssignment(local_assignment) => local_assignment.format(f),
            Self::Property(property_assignment) => property_assignment.format(f),
            Self::Expression(expression_statement) => expression_statement.format(f),
            Self::Error(_) => Node::Nil,
        }
    }
}

impl Format for Vec<(ast::Statement, ast::TrailingExtras)> {
    fn format(&self, f: &FormatConfig) -> Node {
        // Join statements with a hardline so they sit on separate lines
        self.iter()
            .map(|(statement, extras)| {
                node!(f =>
                    statement
                    if statement.ends_with_semicolon() { node!(';' Node::Softline) } else { Node::Nil }
                    extras
                    Node::AdditionalIndent(0)
                )
            })
            .collect::<Vec<Node>>()
            .into()
    }
}

impl Format for ast::StatementList {
    fn format(&self, f: &FormatConfig) -> Node {
        node!(f, self.extras =>
            match (self.statements.is_empty(), &self.tail) {
                (true, Some(tail)) => tail.format(f),
                (false, Some(tail)) => {
                    node!(f =>
                        self.statements
                        tail
                    )
                }
                (false, None) => self.statements.format(f),
                (true, None) => Node::Nil,
            }
        )
    }
}
