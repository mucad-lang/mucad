// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mdbook `book.toml` Configuraion file

use toml_edit::DocumentMut;

use crate::MdBookError;

/// Wrapper around `book.toml` using format-preserving TOML AST.
#[derive(Debug, Clone)]
pub struct MdBookConfig {
    pub doc: DocumentMut,
}

/// Builder methods
impl MdBookConfig {}

/// Accessors
impl MdBookConfig {
    /// Load `book.toml` from a path.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, MdBookError> {
        Ok(Self {
            doc: std::fs::read_to_string(path)?.parse()?,
        })
    }

    // --- Read Accessors ---

    /// Returns `book.title` if present.
    pub fn title(&self) -> Option<&str> {
        self.doc.get("book")?.get("title")?.as_str()
    }

    /// Returns `book.authors` as a list of string slices.
    pub fn authors(&self) -> Vec<&str> {
        self.doc
            .get("book")
            .and_then(|b| b.get("authors"))
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default()
    }

    /// Returns `book.src`, defaulting to `"src"` per mdbook standard.
    pub fn src_path(&self) -> std::path::PathBuf {
        self.doc
            .get("book")
            .and_then(|b| b.get("src"))
            .and_then(|s| s.as_str())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("src"))
    }

    /// Returns `book.language` if present (e.g. `"en"`).
    pub fn language(&self) -> Option<&str> {
        self.doc.get("book")?.get("language")?.as_str()
    }

    // --- Mutators (Lossless) ---

    /// Update or insert `book.title` without removing surrounding comments.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.ensure_book_table();
        self.doc["book"]["title"] = toml_edit::value(title.into());
    }

    /// Update or insert `book.src`.
    pub fn set_src_path(&mut self, src: impl AsRef<std::path::Path>) {
        self.ensure_book_table();
        self.doc["book"]["src"] = toml_edit::value(src.as_ref().to_string_lossy().to_string());
    }

    /// Helper to ensure the `[book]` table exists before inserting keys.
    fn ensure_book_table(&mut self) {
        if !self.doc.contains_key("book") {
            self.doc["book"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
    }
}

impl std::fmt::Display for MdBookConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.doc.fmt(f)
    }
}

impl Default for MdBookConfig {
    fn default() -> Self {
        // Embed and parse the default book.toml template at compile time.
        // Adjust the relative path to wherever your default template file lives.
        let doc = include_str!("book.toml")
            .parse::<DocumentMut>()
            .expect("embedded default book.toml must be valid TOML");

        Self { doc }
    }
}
