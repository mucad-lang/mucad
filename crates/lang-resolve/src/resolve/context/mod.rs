// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cache;
mod lib_ctx;
mod loader;

pub use cache::{LibraryCache, SourceCache};
pub use lib_ctx::ResolveLibraryContext;
use microcad_lang_base::{HashId, SharedMut};
use microcad_lang_lower::Ir;

#[derive(Debug, Default)]
pub struct ResolveContext {
    _lib_search_paths: Vec<std::path::PathBuf>,
    lib_cache: SharedMut<LibraryCache>,
    src_cache: SharedMut<SourceCache>,
}

/// Builder methods.
impl ResolveContext {
    /// Create a `ResolveContext` with default search paths and fresh caches.
    pub fn new() -> Self {
        ResolveContextBuilder::new().build()
    }

    /// Create a builder for customized resolver setup.
    pub fn builder() -> ResolveContextBuilder {
        ResolveContextBuilder::new()
    }
}

/// Accessors
impl ResolveContext {
    /// Get intermediate representation by hash id.
    pub fn get_ir(&self, id: HashId) -> Option<Ir> {
        self.src_cache
            .read_unwrap()
            .get(id)
            .and_then(|source_unit| source_unit.ir().cloned())
    }

    /// Source cache
    pub fn source_cache(&self) -> SharedMut<SourceCache> {
        self.src_cache.clone()
    }
}

/// Builder for constructing a [`ResolveContext`].
#[derive(Debug, Default)]
pub struct ResolveContextBuilder {
    lib_search_paths: Vec<std::path::PathBuf>,
    lib_cache: Option<SharedMut<LibraryCache>>,
    src_cache: Option<SharedMut<SourceCache>>,
}

impl ResolveContextBuilder {
    /// Create a new `ResolveContextBuilder` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single library search path.
    pub fn with_search_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.lib_search_paths.push(path.into());
        self
    }

    /// Add multiple library search paths.
    pub fn with_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<std::path::PathBuf>,
    {
        self.lib_search_paths
            .extend(paths.into_iter().map(Into::into));
        self
    }

    /// Attach a shared library cache.
    pub fn with_lib_cache(mut self, lib_cache: SharedMut<LibraryCache>) -> Self {
        self.lib_cache = Some(lib_cache);
        self
    }

    /// Attach a shared source cache.
    pub fn with_src_cache(mut self, src_cache: SharedMut<SourceCache>) -> Self {
        self.src_cache = Some(src_cache);
        self
    }

    /// Build and initialize the [`ResolveContext`].
    pub fn build(self) -> ResolveContext {
        ResolveContext {
            _lib_search_paths: self.lib_search_paths,
            lib_cache: self.lib_cache.unwrap_or_default(),
            src_cache: self.src_cache.unwrap_or_default(),
        }
    }
}
