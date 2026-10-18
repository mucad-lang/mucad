// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod markdown;
mod mdbook;
mod source_file;
mod stdin;

use derive_more::From;

use mucad_lang_base::{DiagRenderOptions, SourceKind, Url};
pub use source_file::SourceFile;
pub use stdin::Stdin;

use crate::prelude as mu;

pub type Markdown = markdown::MarkdownDocument;
pub type MdBook = mdbook::MdBookDocument;

/// A document containing µcad code.
#[derive(From)]
pub enum Document {
    /// A single source file
    SourceFile(Box<SourceFile>),

    /// A markdown file containing source code snippets
    Markdown(Markdown),

    /// An `book.toml` of a markdown book
    MdBook(MdBook),
}

impl Document {
    /// Create a new document
    ///
    /// If the URL ends with:
    /// * `.µcad`/`.mcad`/`.ucad`: Create a source file
    /// * `.md`: Create a markdown
    /// * `book.toml`: Create an MdBook
    ///
    /// If the URLs scheme is `builtin`, create a built-in document.
    pub fn new(url: Url) -> mu::Result<Self> {
        let path = url.path();
        if path.ends_with("/book.toml") {
            Ok(MdBook::new(url)?.into())
        } else if path.ends_with(".md") {
            Ok(Markdown::new(SourceKind::from(url))?.into())
        } else if url.scheme() == "file" {
            Ok(Box::new(SourceFile::load_from_file(url)?).into())
        } else {
            Err(miette::miette!("Invalid document type: {}", url.path()))
        }
    }

    /// Open a document from a location str.
    pub fn open(location: impl AsRef<str>) -> mu::Result<Self> {
        Self::new(mu::locate::to_url(location.as_ref())?)
    }
}

impl mu::commands::GetCode for Document {
    fn get_code(&self) -> Option<&str> {
        match self {
            Self::SourceFile(source_file) => source_file.get_code(),
            Self::Markdown(_) | Self::MdBook(_) => None,
        }
    }
}

impl mu::commands::SetCode for Document {
    fn set_code(&mut self, new_code: String) -> Option<&str> {
        match self {
            Self::SourceFile(source_file) => source_file.set_code(new_code),
            Self::Markdown(_) | Self::MdBook(_) => None,
        }
    }
}

impl mu::commands::Compile for Document {}

impl mu::commands::Format for Document {
    fn format(&mut self, params: &mu::commands::FormatParameters) -> mu::Result<bool> {
        match self {
            Document::SourceFile(item) => item.format(params),
            Document::Markdown(item) => item.format(params),
            Document::MdBook(item) => item.format(params),
        }
    }
}

impl mu::commands::Sync for Document {
    fn sync(&self) -> mu::Result {
        match &self {
            Document::SourceFile(source) => source.sync(),
            Document::Markdown(markdown) => markdown.sync(),
            Document::MdBook(mdbook) => mdbook.sync(),
        }
    }
}

impl mu::commands::PrintDiagnostics for Document {
    fn print_diagnostics(
        &self,
        f: &mut dyn std::fmt::Write,
        options: &DiagRenderOptions,
    ) -> std::fmt::Result {
        match self {
            Document::SourceFile(i) => i.print_diagnostics(f, options),
            Document::Markdown(_) => todo!(),
            Document::MdBook(_) => todo!(),
        }
    }
}
