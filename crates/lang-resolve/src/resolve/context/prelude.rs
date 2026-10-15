// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Symbols which can be used by name without `use`.

use microcad_lang_base::{HashMap, Identifier, LibraryId, SymbolId};

/// Symbols of a library which can be used by name without `use`,
/// e.g. `Circle` for `std::geo2d::Circle`.
///
/// Every other symbol with the same name has priority over a symbol of the prelude.
#[derive(Debug, Clone)]
pub struct Prelude {
    /// The library of the symbols, e.g. `std`. Only libraries depending on it use the prelude.
    pub library: LibraryId,
    /// The symbols by name.
    pub symbols: HashMap<Identifier, SymbolId>,
}

impl Prelude {
    /// An empty prelude of `library`.
    pub fn new(library: LibraryId) -> Self {
        Self {
            library,
            symbols: HashMap::default(),
        }
    }
}
