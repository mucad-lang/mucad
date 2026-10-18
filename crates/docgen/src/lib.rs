// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad documentation generators

mod generators;
mod mdbook;

pub use generators::{BuiltinLibraryDocGenerator, LibraryDocGenerator};
use mucad_lang_markdown::{Markdown, MdBook, WriteToFile};

use std::error::Error;

/// Generated documentation, which can be written to a path.
pub trait WriteDoc {
    fn write(&self, path: &std::path::Path) -> Result<(), Box<dyn Error>>;
}

/// An mdbook is written into a directory.
impl WriteDoc for MdBook {
    fn write(&self, path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        Ok(self.save(path)?)
    }
}

/// A single markdown is written into a file.
impl WriteDoc for Markdown {
    fn write(&self, path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        Ok(self.write_to_file(path)?)
    }
}

/// Documentation generator trait.
pub trait GenerateDoc<Output: WriteDoc> {
    fn generate_doc(&self) -> Result<Output, Box<dyn Error>>;

    /// Generate the documentation and write it to `path`.
    fn write_doc(&self, path: impl AsRef<std::path::Path>) -> Result<(), Box<dyn Error>> {
        self.generate_doc()?.write(path.as_ref())
    }
}
