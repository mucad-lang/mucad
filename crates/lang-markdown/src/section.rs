// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Markdown section

use crate::Block;

/// A markdown section with heading and block elements.
#[derive(Debug, Clone, Default)]
pub struct Section {
    /// A heading, displayed with a `#` prefix in markdown.
    pub heading: String,
    /// Section level.
    pub level: i64,
    /// The section content, consisting of blocks.
    pub content: Vec<Block>,
}

impl Section {
    /// A section with `n` levels deeper.
    pub fn nested(&self, n: i64) -> Section {
        Section {
            heading: self.heading.clone(),
            level: self.level + n,
            content: self.content.clone(),
        }
    }

    /// Returns `false` if this section has a heading and a content.
    pub fn is_empty(&self) -> bool {
        self.heading.is_empty() && self.content.is_empty()
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Level 0 is the content before the first heading.
        if self.level > 0 {
            writeln!(f, "{} {}", "#".repeat(self.level as usize), self.heading)?;
            if !self.content.is_empty() {
                writeln!(f)?;
            }
        }
        if self.content.is_empty() {
            Ok(())
        } else {
            self.content.iter().enumerate().try_for_each(|(i, block)| {
                if i > 0 {
                    writeln!(f)?;
                }
                write!(f, "{block}")
            })
        }
    }
}
