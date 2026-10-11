// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Markdown book generators for built-in and compiled libraries.

mod builtin;
mod library;

use microcad_lang_markdown::{Block, Markdown};

/// Trait to fetch markdown from a symbol definition.
pub(crate) trait ToMd {
    fn to_md(&self) -> Markdown;
}

/// Documentation text as written, if there is any.
pub(crate) fn doc_block(doc: Option<&str>) -> Option<Block> {
    doc.map(str::trim)
        .filter(|doc| !doc.is_empty())
        .map(|doc| Block::Paragraph(doc.to_string()))
}
