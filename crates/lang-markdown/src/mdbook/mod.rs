// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad markdown book support.

mod config;
mod summary;

use microcad_lang_base::{HashMap, WriteToFile};
use miette::Diagnostic;
use thiserror::Error;

pub use config::MdBookConfig;
pub use summary::{MdBookSummary, MdBookSummaryEntry};

use crate::{CodeBlock, Markdown, MarkdownError};

#[derive(Debug, Error, Diagnostic)]
pub enum MdBookError {
    /// Io Error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Toml Error
    #[error("TOML error: {0}")]
    TomlEdit(#[from] toml_edit::TomlError),

    /// The directory does not contain an mdbook.
    #[error("No mdbook in directory: {0}")]
    NoMdBookDirectory(std::path::PathBuf),

    #[error("Error saving markdown file `{file}`: {err}")]
    Save {
        file: std::path::PathBuf,
        err: MarkdownError,
    },
}

/// Directory that contains a markdown book.
pub struct MdBook {
    /// Markdown files, with paths relative to the source folder of the book (e.g. `src`).
    pub md_files: HashMap<std::path::PathBuf, Markdown>,

    /// Book.toml file
    pub config: MdBookConfig,
}

impl MdBook {
    pub fn new(config: MdBookConfig) -> Self {
        Self {
            config,
            md_files: HashMap::default(),
        }
    }

    /// Load an MdBook from file.
    ///
    /// Will fail if the directory does not contain a `book.toml` file.
    /// Scans the directory `src` recursively for markdown files ending with `.md`.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, MdBookError> {
        let root = path.as_ref();

        let root = if root.ends_with("book.toml") {
            root.parent()
                .map(|path| path.to_path_buf())
                .unwrap_or(std::env::current_dir()?)
        } else {
            root.to_path_buf()
        };

        let config_path = root.join("book.toml");

        // 1. Validate book.toml existence
        if !config_path.exists() {
            return Err(MdBookError::NoMdBookDirectory(root));
        }

        let config = MdBookConfig::load(config_path)?;

        // 2. Identify the src directory
        let src_path = root.join("src");
        let mut md_files = Vec::new();

        // 3. Recursively scan src for .md files
        if src_path.exists() && src_path.is_dir() {
            Self::visit_dirs(&src_path, &src_path, &mut md_files);
        }

        let md_files = md_files
            .iter()
            .map(|md_file| {
                (
                    md_file.clone(),
                    Markdown::load(src_path.join(md_file))
                        .unwrap_or_else(|_| panic!("No error: {}", md_file.display())),
                )
            })
            .collect();

        Ok(Self { config, md_files })
    }

    /// Add a new markdown file to the mdbook.
    ///
    /// The path is relative to the source folder of the book, e.g. `SUMMARY.md` or `geo2d/README.md`.
    pub fn add_md(&mut self, path: impl AsRef<std::path::Path>, md: impl Into<Markdown>) {
        self.md_files.insert(path.as_ref().to_path_buf(), md.into());
    }

    pub fn abs_md_file_path(&self, md_file: impl AsRef<std::path::Path>) -> std::path::PathBuf {
        self.config.src_path().join(md_file.as_ref())
    }

    /// Write the book into directory `path`: `book.toml` and all markdown files in the source folder.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), MdBookError> {
        let root = path.as_ref();
        self.config.write_to_file(root.join("book.toml"))?;

        let src_path = root.join(self.config.src_path());
        self.md_files.iter().try_for_each(|(md_file_path, md)| {
            let md_file_path = src_path.join(md_file_path);

            md.write_to_file(&md_file_path)
                .map_err(|err| MdBookError::Save {
                    file: md_file_path.clone(),
                    err: crate::markdown::MarkdownError::IoError(err),
                })
        })
    }

    /// Returns an iterator over all code blocks in the entire document.
    pub fn code_blocks(&self) -> impl Iterator<Item = (std::path::PathBuf, &CodeBlock)> {
        self.md_files.iter().flat_map(|(md_file, md)| {
            md.code_blocks()
                .map(|code_block| (md_file.clone(), code_block))
        })
    }

    /// Returns an iterator over all code blocks in the entire document.
    pub fn code_blocks_mut(
        &mut self,
    ) -> impl Iterator<Item = (std::path::PathBuf, &mut CodeBlock)> {
        self.md_files.iter_mut().flat_map(|(md_file, md)| {
            md.code_blocks_mut()
                .map(|code_block| (md_file.clone(), code_block))
        })
    }

    /// Helper to recursively find markdown files.
    ///
    /// Stores paths relative to the `src` folder.
    fn visit_dirs(
        src_root: &std::path::Path,
        dir: &std::path::Path,
        cb: &mut Vec<std::path::PathBuf>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::visit_dirs(src_root, &path, cb);
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    // Strip the src_root prefix to keep paths relative to src
                    if let Ok(relative) = path.strip_prefix(src_root) {
                        cb.push(relative.to_path_buf());
                    }
                }
            }
        }
    }
}
