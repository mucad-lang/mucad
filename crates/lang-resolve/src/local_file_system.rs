// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The local file system.

use mucad_lang_base::FileSystem;

/// The local file system, from which sources and libraries are loaded by default.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn is_file(&self, path: &std::path::Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &std::path::Path) -> bool {
        path.is_dir()
    }

    fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn canonicalize(&self, path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
        std::fs::canonicalize(path)
    }
}
