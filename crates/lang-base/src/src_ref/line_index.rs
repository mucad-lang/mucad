// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;

use crate::{LineCol, Source};

/// An index to retrieve the offsets in a line in O(log(n)).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LineIndex {
    /// Offset (bytes) the beginning of each line, zero-based
    line_offsets: Vec<u32>,
}

impl LineIndex {
    /// Create a new line index from a &str.
    pub fn new(s: &str, line_offset: u32) -> Self {
        Self {
            line_offsets: std::iter::once(0)
                .chain(
                    s.match_indices('\n')
                        .map(|(i, _)| (i + 1) as u32 + line_offset),
                )
                .collect(),
        }
    }

    /// Returns (line, col) of pos.
    ///
    /// The pos is a byte offset, start from 0, e.g. "ab" is 2, "你好" is 6
    pub fn line_col(&self, input: &str, pos: usize) -> LineCol {
        let line = self.line_offsets.partition_point(|&it| it <= pos as u32) - 1;
        let first_offset = self.line_offsets[line] as usize;

        // Get line str from original input, then we can get column offset
        let line_str = &input[first_offset..pos];
        let col = line_str.chars().count();

        LineCol {
            line: line as u32,
            col: (col + 1) as u32,
        }
    }
}

impl<'source> From<&'source Source> for LineIndex {
    fn from(source: &'source Source) -> Self {
        Self::new(
            source.code(),
            source.location.line_offset.unwrap_or_default(),
        )
    }
}
