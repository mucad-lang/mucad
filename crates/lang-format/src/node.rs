// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{FormatConfig, node};

use mucad_lang_base::{CompactString, ToCompactString};

/// How to perform a line break
pub enum BreakMode {
    NoBreak,
    WithIndent(usize),
}

impl BreakMode {
    /// Determines the break strategy based on the format config
    pub fn from_layout(nodes: &[Node], max_items: usize, f: &FormatConfig) -> Self {
        let width: usize = nodes.iter().map(|node| node.estimate_width()).sum();
        let too_many_items = max_items > 0 && nodes.len() > max_items;
        let too_wide = width > f.max_width;
        let forced_break = nodes.iter().any(|node| node.ends_with_hardline());

        if too_many_items || too_wide || forced_break {
            Self::WithIndent(f.indent_width)
        } else {
            Self::NoBreak
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Node {
    #[default]
    Nil,
    Text(CompactString),
    // A comment starting with `//`
    SingleLineComment(CompactString),
    Hardline,
    AdditionalIndent(usize),
    Softline,
    Indent {
        width: usize,
        node: Box<Node>,
    },
    Group(Vec<Node>),
}

impl Node {
    /// Intersperses a separator between nodes from any iterable source.
    pub fn hlist<I>(nodes: I, separator: impl Into<Node>) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let iter = nodes.into_iter();
        let separator: Node = separator.into();
        // Provide a hint to the allocator if possible
        let (lower, _) = iter.size_hint();
        let mut result = Vec::with_capacity(lower.saturating_mul(2));

        let mut first = true;
        for node in iter {
            if !first {
                result.push(separator.clone());
            }
            result.push(node);
            first = false;
        }

        result.into()
    }

    pub fn remove_hardline(self) -> Node {
        match self {
            Node::Nil | Node::Softline | Node::AdditionalIndent(_) | Node::SingleLineComment(_) => {
                self.clone()
            }
            Node::Text(compact_string) => compact_string.trim_end().into(),
            Node::Hardline => Node::Nil,
            Node::Indent { width, node } => Node::Indent {
                width,
                node: Box::new(node.remove_hardline()),
            },
            Node::Group(mut group) => {
                // Check if the last element exists and is a Hardline
                if let Some(Node::Hardline) = group.last() {
                    group.pop(); // Remove it
                }

                // If you need to recursively remove hardlines from the new last element,
                // you could call .remove_hardline() on the group itself or the new tail,
                // but for a simple "pop if last", this is sufficient:
                Node::Group(group)
            }
        }
    }

    pub fn vlist<I>(nodes: I, separator: impl Into<Node>, indent_width: usize) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let sep = separator.into();
        let nodes: Node = nodes
            .into_iter()
            .map(|node| node!(node.remove_hardline() sep.clone() Node::Hardline))
            .collect::<Vec<_>>()
            .into();

        match indent_width {
            0 => nodes,
            width => node!(
                Node::Hardline
                Node::Indent { width, node: Box::new(nodes) }
            ),
        }
    }

    /// A list of items with an separator
    pub fn list<I>(nodes: I, separator: impl Into<Node>, break_mode: BreakMode) -> Node
    where
        I: IntoIterator<Item = Node>,
    {
        let sep = separator.into();
        match break_mode {
            BreakMode::NoBreak => Self::hlist(nodes, node!(sep Node::Softline)),
            BreakMode::WithIndent(indent_width) => Self::vlist(nodes, sep, indent_width),
        }
    }

    pub fn estimate_width(&self) -> usize {
        match &self {
            Node::Nil => 0,
            Node::Text(compact_string) => compact_string.len(),
            Node::Hardline | Node::SingleLineComment(_) => 0,
            Node::AdditionalIndent(width) => *width,
            Node::Softline => 1,
            Node::Indent { width, node } => width + node.estimate_width(),
            Node::Group(group) => group
                .iter()
                .map(|node| node.estimate_width())
                .sum::<usize>(),
        }
    }

    pub fn indent(width: usize, node: impl Into<Node>) -> Self {
        Node::Indent {
            width,
            node: Box::new(node.into()),
        }
    }

    pub fn contains_hardline(&self) -> bool {
        match &self {
            Node::Nil | Node::Softline | Node::AdditionalIndent(_) => false,
            Node::Text(compact_string) => compact_string.contains("\n"),
            Node::Hardline | Node::SingleLineComment(_) => true,
            Node::Indent { width: _, node } => node.contains_hardline(),
            Node::Group(group) => group.iter().any(|node| node.contains_hardline()),
        }
    }

    pub fn ends_with_hardline(&self) -> bool {
        match &self {
            Node::Nil | Node::Softline | Node::AdditionalIndent(_) => false,
            Node::Text(compact_string) => compact_string.ends_with("\n"),
            Node::Hardline | Node::SingleLineComment(_) => true,
            Node::Indent { width: _, node } => node.ends_with_hardline(),
            Node::Group(group) => group.iter().any(|node| node.ends_with_hardline()),
        }
    }

    pub fn starts_with_hardline(&self) -> bool {
        match &self {
            Node::Nil | Node::Softline | Node::AdditionalIndent(_) => false,
            Node::Text(compact_string) => compact_string.starts_with("\n"),
            Node::Hardline | Node::SingleLineComment(_) => true,
            Node::Indent { width: _, node } => node.starts_with_hardline(),
            Node::Group(group) => group.iter().any(|node| node.starts_with_hardline()),
        }
    }

    /// Compact nodes: Remove Nil nodes and concatenate adjacent Text ndoes
    pub fn compact(nodes: Vec<Node>) -> Self {
        // 1. Filter out Nil and
        // 2. flatten nested Groups
        // 3. Merge adjacent Text nodes
        let mut compacted = Vec::with_capacity(nodes.len());

        for node in nodes {
            match (compacted.last_mut(), node) {
                (_, Node::Nil) => continue,
                // Merge adjacent text nodes into one
                (Some(Node::Text(last)), Node::Text(next)) => {
                    last.push_str(&next);
                }
                (Some(Node::Softline), Node::Softline) => continue,
                (_, Node::Group(group)) => {
                    let flattened = Self::compact(group);
                    match flattened {
                        Node::Group(mut sub_vec) => compacted.append(&mut sub_vec),
                        Node::Nil => continue,
                        node => {
                            if let (Some(Node::Text(last)), Node::Text(next)) =
                                (compacted.last_mut(), &node)
                            {
                                last.push_str(next);
                            } else {
                                compacted.push(node);
                            }
                        }
                    }
                }
                // Otherwise, add the node
                (_, other) => compacted.push(other),
            }
        }

        // 4. Handle the resulting length
        match compacted.len() {
            0 => Node::Nil,
            1 => compacted.remove(0),
            _ => Node::Group(compacted),
        }
    }
}

impl From<Vec<Node>> for Node {
    fn from(nodes: Vec<Node>) -> Self {
        Node::compact(nodes)
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::Text(value.to_compact_string())
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Text(value.to_compact_string())
    }
}

impl From<char> for Node {
    fn from(value: char) -> Self {
        Node::Text(value.to_compact_string())
    }
}

impl From<CompactString> for Node {
    fn from(value: CompactString) -> Self {
        Node::Text(value)
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut state = RenderState::default();
        self.render_recursive(f, &mut state)
    }
}

#[derive(Default)]
struct RenderState {
    indent_level: usize,
    column: usize,
    additional_indent: usize,
    indent_pending: bool,
    softline_pending: bool,
    extra_pending: bool,
}

impl Node {
    fn render_recursive(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        state: &mut RenderState,
    ) -> std::fmt::Result {
        fn write_pending_indent(
            f: &mut std::fmt::Formatter<'_>,
            state: &mut RenderState,
        ) -> std::fmt::Result {
            if state.indent_pending {
                let spaces = " ".repeat(state.indent_level);
                state.indent_pending = false;
                write!(f, "{spaces}")
            } else {
                Ok(())
            }
        }

        fn write_extra_pending(
            f: &mut std::fmt::Formatter<'_>,
            state: &mut RenderState,
        ) -> std::fmt::Result {
            if state.softline_pending {
                state.softline_pending = false;
                state.column += 1;
                if !state.indent_pending {
                    write!(f, " ")?;
                }
            }

            if state.extra_pending {
                state.extra_pending = false;
                state.column = state.indent_level;
                writeln!(f)
            } else {
                Ok(())
            }
        }

        match self {
            Node::Text(s) => {
                write_extra_pending(f, state)?;
                write_pending_indent(f, state)?;
                state.column += s.len();
                write!(f, "{s}")
            }
            Node::SingleLineComment(s) => {
                write_extra_pending(f, state)?;
                write_pending_indent(f, state)?;
                state.column += s.len();
                state.indent_pending = true;
                state.extra_pending = true;
                write!(f, "{s}")
            }
            Node::Hardline => {
                state.column = state.indent_level;
                state.indent_pending = true;
                writeln!(f)
            }
            Node::Softline => {
                state.softline_pending =
                    state.column as i32 - state.indent_level as i32 > 0 && !state.indent_pending;
                Ok(())
            }
            Node::Group(group) => {
                write_extra_pending(f, state)?;
                write_pending_indent(f, state)?;
                group
                    .iter()
                    .try_for_each(|node| node.render_recursive(f, state))
            }
            Node::Indent { width, node } => {
                state.indent_level += width + state.additional_indent;
                node.render_recursive(f, state)?;
                state.indent_level -= width + state.additional_indent;
                state.additional_indent = 0;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
