// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Virtual file system.

use crate::fs::FileSystem;

/// A file system in memory, e.g. in a browser without a local file system.
///
/// Directories exist implicitly, if they contain files.
#[derive(Debug, Default, Clone)]
pub struct VirtualFileSystem {
    /// The content of the files by their path.
    files: std::collections::HashMap<std::path::PathBuf, String>,
}

impl VirtualFileSystem {
    /// An empty virtual file system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file with `content` at `path`, an existing file is replaced.
    pub fn insert(&mut self, path: impl Into<std::path::PathBuf>, content: impl Into<String>) {
        self.files.insert(path.into(), content.into());
    }

    /// Builder method to add a file with `content` at `path`.
    pub fn with_file(
        mut self,
        path: impl Into<std::path::PathBuf>,
        content: impl Into<String>,
    ) -> Self {
        self.insert(path, content);
        self
    }

    /// The error for a missing `path`.
    fn not_found(path: &std::path::Path) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::NotFound, path.display().to_string())
    }
}

impl FileSystem for VirtualFileSystem {
    fn is_file(&self, path: &std::path::Path) -> bool {
        self.files.contains_key(path)
    }

    fn is_dir(&self, path: &std::path::Path) -> bool {
        self.files
            .keys()
            .any(|file| file != path && file.starts_with(path))
    }

    fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| Self::not_found(path))
    }

    /// Paths are absolute already, e.g. `/lib/std/lib.mu`.
    fn canonicalize(&self, path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
        match self.exists(path) {
            true => Ok(path.to_path_buf()),
            false => Err(Self::not_found(path)),
        }
    }
}
