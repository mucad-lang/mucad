// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{OrderedList, UnorderedList, code_block::CodeBlock};

/// A block quote. Each line starts with `>`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockQuote(Vec<String>);

fn stripped_prefix<T: IntoIterator<Item = String>>(iter: T, prefix: &str) -> Vec<String> {
    iter.into_iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix(prefix)?;
            Some(rest.strip_prefix(' ').unwrap_or(rest).to_string())
        })
        .collect()
}

impl BlockQuote {
    const PREFIX: &'static str = ">";
}

impl std::fmt::Display for BlockQuote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|line| writeln!(f, "{prefix} {line}", prefix = Self::PREFIX))
    }
}

impl FromIterator<String> for BlockQuote {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(stripped_prefix(iter, Self::PREFIX))
    }
}

/// A table in markdown. Each line starts with `|`.
#[derive(Debug, Clone, PartialEq)]
pub struct Table(Vec<String>);

impl Table {
    const PREFIX: &'static str = "|";
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|line| writeln!(f, "{prefix} {line}", prefix = Self::PREFIX))
    }
}

impl FromIterator<String> for Table {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(stripped_prefix(iter, Self::PREFIX))
    }
}

/// A block in a section. Each block ends with a new line.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A paragraph with text.
    Paragraph(String),

    /// A µcad code block starting with ```µcad or with `[![test](...)` banner.
    CodeBlock(CodeBlock),

    /// A table. Each line starts with `|`.
    Table(Table),

    /// A block quote. Each line starts with `>`.
    BlockQuote(BlockQuote),

    /// A list with bullet points. Each point starts with `-`. Nested items are indented.
    UnorderedList(UnorderedList),

    /// A numbered list. Each point starts with a number, e.g. `1.`. Nested items are indented.
    OrderedList(OrderedList),
}

impl Block {
    /// Returns true if the block is a list.
    pub fn is_list(&self) -> bool {
        matches!(self, Block::UnorderedList(_) | Block::OrderedList(_))
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Block::Paragraph(text) => writeln!(f, "{text}"),
            Block::CodeBlock(code_block) => writeln!(f, "{code_block}"),
            // These already end each line with a new line.
            Block::Table(table) => write!(f, "{table}"),
            Block::BlockQuote(block_quote) => write!(f, "{block_quote}"),
            Block::UnorderedList(list) => write!(f, "{list}"),
            Block::OrderedList(list) => write!(f, "{list}"),
        }
    }
}
