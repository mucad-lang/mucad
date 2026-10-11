// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language markdown support library.

mod block;
mod code_block;
mod error;
mod list;
mod markdown;
mod mdbook;
mod section;
pub mod table_builder;

pub use block::Block;
pub use code_block::{CodeBlock, CodeBlockHeader};
pub use error::ParseError;
pub use list::{ListItem, OrderedList, UnorderedList};
pub use markdown::{Markdown, MarkdownError};
pub use mdbook::{MdBook, MdBookError, MdBookSummary, MdBookSummaryEntry, MdbookConfig};
pub use section::Section;

/// Helper macro to construct a Markdown from any format string.
#[macro_export]
macro_rules! md {
    ($($arg:tt)*) => {
        format!($($arg)*).parse::<$crate::Markdown>().unwrap_or_default()
    };
}

pub use microcad_lang_base::WriteToFile;
