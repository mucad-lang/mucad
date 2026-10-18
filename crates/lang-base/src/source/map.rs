// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use mucad_hash::{HashId, HashMap};

use crate::{Source, SourceLocation};

#[derive(Debug, Default, Clone)]
pub struct SourceMap {
    /// Maps the content hash directly to the loaded Source tree
    by_hash_id: HashMap<HashId, Arc<Source>>,
    /// Maps locations (paths/URLs) to the current active hash for that location
    by_location: HashMap<SourceLocation, HashId>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns a source file into the map based purely on its content hash.
    pub fn insert(&mut self, source: impl Into<Source>) -> HashId {
        let source = source.into();
        let hash_id = source.hash_id();
        let location = source.location.clone();

        self.by_hash_id
            .entry(hash_id)
            .or_insert_with(|| Arc::new(source));
        self.by_location.insert(location, hash_id);

        hash_id
    }

    /// Get a source by hash id
    pub fn get_by_hash_id(&self, id: HashId) -> Option<Arc<Source>> {
        self.by_hash_id.get(&id).cloned()
    }

    /// Get a source by its location
    pub fn get_by_location(&self, location: impl Into<SourceLocation>) -> Option<Arc<Source>> {
        let location = location.into();

        self.by_location
            .get(&location)
            .and_then(|hash_id| self.get_by_hash_id(*hash_id))
    }
}
