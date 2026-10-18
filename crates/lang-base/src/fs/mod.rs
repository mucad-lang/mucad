// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! File systems, from which sources and libraries are loaded.

mod vfs;

pub use vfs::VirtualFileSystem;

use crate::{Source, SourceLocation};

/// A file system, from which sources and libraries are loaded,
/// e.g. the local one or a virtual one in a browser.
pub trait FileSystem: Send + Sync + std::fmt::Debug {
    /// Returns `true` if `path` is an existing file.
    fn is_file(&self, path: &std::path::Path) -> bool;

    /// Returns `true` if `path` is an existing directory.
    fn is_dir(&self, path: &std::path::Path) -> bool;

    /// Read the file at `path` into a string.
    fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String>;

    /// The absolute path of the existing `path`.
    fn canonicalize(&self, path: &std::path::Path) -> std::io::Result<std::path::PathBuf>;

    /// Returns `true` if `path` is an existing file or directory.
    fn exists(&self, path: &std::path::Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    /// Load the source of the file at `path`.
    fn load_source(&self, path: &std::path::Path) -> std::io::Result<Source> {
        let code = self.read_to_string(path)?;
        Ok(Source::new(SourceLocation::new(path.to_path_buf()), code))
    }
}
