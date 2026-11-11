// Copyright © 2026 The µcad authors <info@mucad.org>
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
pub use mdbook::{MdBook, MdBookConfig, MdBookError, MdBookSummary, MdBookSummaryEntry};
pub use section::Section;

/// This is a helper function to convert a path into string.
/// It always uses `/` as separator, regardless of the platform.
/// This hack is need because Windows uses `\\` as directory separator instead of `/` like 'normal' OSes.
pub fn display_path(path: impl AsRef<std::path::Path>) -> String {
    path.as_ref().display().to_string().replace('\\', "/")
}

/// Helper macro to construct a Markdown from any format string.
#[macro_export]
macro_rules! md {
    ($($arg:tt)*) => {
        format!($($arg)*).parse::<$crate::Markdown>().unwrap_or_default()
    };
}

pub use mucad_lang_base::WriteToFile;
