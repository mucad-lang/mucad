// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Access to the modules of other libraries in the library cache.

use std::cell::RefCell;

use mucad_lang_base::{ExternalId, HashSet, Identifier, SharedMut, SymbolId};

use crate::{
    LibraryCache,
    library::{External, Search},
};

/// Looks up and lists symbols inside modules of other libraries in the library cache.
///
/// Glob imports of these modules are followed into further libraries, e.g. a module of
/// library `a` which re-exports a module of library `b` with `pub use b::geo::*;`.
pub(crate) struct ExternalResolver {
    lib_cache: SharedMut<LibraryCache>,
    /// The modules being searched, to stop at libraries which import each other.
    searching: RefCell<HashSet<ExternalId>>,
}

impl ExternalResolver {
    pub(crate) fn new(lib_cache: SharedMut<LibraryCache>) -> Self {
        Self {
            lib_cache,
            searching: Default::default(),
        }
    }

    /// Run `f` with the library of `module`, unless `module` is being searched already.
    fn search<T: Default>(&self, module: &ExternalId, f: impl FnOnce(&crate::Library) -> T) -> T {
        if !self.searching.borrow_mut().insert(module.clone()) {
            return T::default();
        }
        let lib = self.lib_cache.read_unwrap().get(&module.lib_id);
        let result = lib.map(|lib| f(&lib)).unwrap_or_default();
        self.searching.borrow_mut().remove(module);
        result
    }
}

impl External for ExternalResolver {
    fn look_up(&self, module: &ExternalId, parts: &[Identifier]) -> Option<SymbolId> {
        self.search(module, |lib| {
            lib.look_up_in_module(module.id, parts, self)
                .map(|found| found.symbol_id(lib, Some(module.lib_id)))
        })
    }

    fn symbols(&self, module: &ExternalId) -> Vec<(Identifier, SymbolId)> {
        self.search(module, |lib| {
            // From the root, only public symbols inside the module are visible.
            Search::new(lib, lib.root, self)
                .symbols_in(module.id)
                .into_iter()
                .map(|(name, found)| (name, found.symbol_id(lib, Some(module.lib_id))))
                .collect()
        })
    }
}
