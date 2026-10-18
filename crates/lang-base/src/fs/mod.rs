// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod vfs;

use std::sync::Arc;

use crate::{Source, SourceLocation};

pub use vfs::VirtualFileSystem;

/// An abstraction layer for file system
pub trait FileSystem: Send + Sync {
    /// Eagerly loads a resource and wraps it into a complete Source type.
    /// Returning an Arc lets you share this across modules without cloning strings.
    fn load_source(&mut self, location: SourceLocation) -> Result<Arc<Source>, std::io::Error>;

    /// Write source to file
    fn write_source(&self, _source: &Source) -> Result<(), std::io::Error> {
        unimplemented!()
    }

    fn source_exists(&self, location: &SourceLocation) -> bool;
}
