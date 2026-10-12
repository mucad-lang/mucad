// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::{GetSourceByHash, HashId, HashMap, LibraryId, Shared, Source};

use crate::{Library, ResolveErrorKind, ResolveResult, SourceUnit};

#[derive(Debug, Default)]
pub struct LibraryCache {
    /// Global lookup for all loaded/compiled libraries
    libraries: HashMap<LibraryId, Shared<Library>>,
}

impl LibraryCache {
    /// Insert an external library (a dependency).
    ///
    /// The library must have a manifest, because its name and version define the library ID.
    pub fn insert(&mut self, library: Library) -> ResolveResult<LibraryId> {
        if library.manifest.is_none() {
            return Err(ResolveErrorKind::LibraryWithoutManifest.into());
        }
        let id = library.id();
        self.libraries.insert(id, Shared::from(library));
        Ok(id)
    }

    /// Insert the main library under [`LibraryId::ROOT`], replacing a previous main library.
    ///
    /// The main library does not need a manifest, e.g. if it is a single source file.
    pub fn insert_root(&mut self, library: Library) -> LibraryId {
        self.libraries
            .insert(LibraryId::ROOT, Shared::from(library));
        LibraryId::ROOT
    }

    pub fn get(&self, id: &LibraryId) -> Option<Shared<Library>> {
        self.libraries.get(id).cloned()
    }
}

#[derive(Debug, Default)]
pub struct SourceCache {
    // Source by hash
    by_hash_id: HashMap<HashId, SourceUnit>,
    // Source by paths
    by_path: HashMap<std::path::PathBuf, HashId>,
}

impl GetSourceByHash for SourceCache {
    fn get_source_by_hash(&'_ self, hash: HashId) -> Option<&Source> {
        self.get(hash).map(|source_unit| source_unit.source())
    }
}

impl SourceCache {
    pub fn insert(&mut self, source: impl Into<Source>) -> HashId {
        let source = source.into();
        let id = source.hash_id();
        if let Some(source) = self.get(id) {
            log::debug!("Source unit '{source}' is already loaded");
            return source.id();
        }

        let source_unit = SourceUnit::new(source).parse().lower();
        if let Some(path) = source_unit.source().path() {
            self.by_path.insert(path, id);
        }
        self.by_hash_id.insert(id, source_unit);
        id
    }

    /// Get a source unit by hash.
    pub fn get(&self, id: HashId) -> Option<&SourceUnit> {
        self.by_hash_id.get(&id)
    }

    /// Get a source unit by path.
    pub fn get_by_path(&self, path: impl AsRef<std::path::Path>) -> Option<&SourceUnit> {
        self.by_path
            .get(path.as_ref())
            .and_then(|hash_id| self.get(*hash_id))
    }
}
