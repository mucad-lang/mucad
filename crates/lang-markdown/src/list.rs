// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Markdown lists.

use crate::Block;

/// An item of a list.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListItem {
    /// Text of the item, e.g. `First` for `- First`.
    pub text: String,
    /// Further blocks of the item, e.g. nested lists or paragraphs.
    pub children: Vec<Block>,
}

impl ListItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = Block>) -> Self {
        self.children.extend(children);
        self
    }

    /// Returns true if the item contains blocks other than lists, e.g. a second paragraph.
    fn is_loose(&self) -> bool {
        self.children.iter().any(|child| !child.is_list())
    }

    /// Write the item with a marker, e.g. `- ` or `1. `.
    ///
    /// Further lines and children are indented by the width of the marker.
    fn fmt_with_marker(&self, f: &mut std::fmt::Formatter<'_>, marker: &str) -> std::fmt::Result {
        let indent = " ".repeat(marker.chars().count());
        let mut lines = self.text.lines();
        writeln!(
            f,
            "{}",
            format!("{marker}{}", lines.next().unwrap_or_default()).trim_end()
        )?;
        lines.try_for_each(|line| writeln!(f, "{indent}{line}"))?;

        self.children.iter().try_for_each(|child| {
            // Paragraphs and code are separated by a blank line, nested lists are not.
            if !child.is_list() {
                writeln!(f)?;
            }
            child.to_string().lines().try_for_each(|line| match line {
                "" => writeln!(f),
                line => writeln!(f, "{indent}{line}"),
            })
        })
    }
}

/// Write list items, separated by a blank line if the list is loose.
fn fmt_items(
    f: &mut std::fmt::Formatter<'_>,
    items: &[ListItem],
    markers: impl Iterator<Item = String>,
) -> std::fmt::Result {
    let loose = items.iter().any(ListItem::is_loose);
    items
        .iter()
        .zip(markers)
        .enumerate()
        .try_for_each(|(i, (item, marker))| {
            if loose && i > 0 {
                writeln!(f)?;
            }
            item.fmt_with_marker(f, &marker)
        })
}

/// A list with bullet points: `- item`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnorderedList(pub Vec<ListItem>);

impl UnorderedList {
    pub fn new(items: impl IntoIterator<Item = ListItem>) -> Self {
        Self(items.into_iter().collect())
    }
}

impl std::fmt::Display for UnorderedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_items(f, &self.0, std::iter::repeat("- ".to_string()))
    }
}

/// A numbered list: `1. item`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrderedList {
    /// Number of the first item.
    pub start: u64,
    pub items: Vec<ListItem>,
}

impl OrderedList {
    pub fn new(start: u64, items: impl IntoIterator<Item = ListItem>) -> Self {
        Self {
            start,
            items: items.into_iter().collect(),
        }
    }
}

impl std::fmt::Display for OrderedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_items(f, &self.items, (self.start..).map(|n| format!("{n}. ")))
    }
}
