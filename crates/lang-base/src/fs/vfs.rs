// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Virtual file system.

use crate::{Source, SourceLocation, SourceMap, fs::FileSystem};

/// A virtual system for µcad sources
pub struct VirtualFileSystem {
    /// Loaded sources
    sources: SourceMap,
}

impl FileSystem for VirtualFileSystem {
    fn load_source(
        &mut self,
        location: SourceLocation,
    ) -> Result<std::sync::Arc<Source>, std::io::Error> {
        match location.path() {
            Some(path) => {
                let source = Source::load(path)?;
                let hash_id = self.sources.insert(source);
                self.sources.get_by_hash_id(hash_id).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, location.to_string())
                })
            }
            None => unimplemented!("Error handling"),
        }
    }

    fn source_exists(&self, location: &SourceLocation) -> bool {
        self.sources.get_by_location(location.clone()).is_some()
    }
}
