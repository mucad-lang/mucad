// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::Identifier;

pub struct SymbolAbsPath {
    pub parts: Vec<Identifier>,
}

/// The path with `::` as separator, e.g. `std::geo2d::Circle`.
impl std::fmt::Display for SymbolAbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<_> = self.parts.iter().map(|part| part.as_str()).collect();
        write!(f, "{}", parts.join("::"))
    }
}

impl SymbolAbsPath {
    /// Returns an iterator over references to the path components (from root to leaf).
    pub fn iter(&self) -> std::slice::Iter<'_, Identifier> {
        self.parts.iter()
    }
}

// 1. Enables: for part in &abs_path { ... }
impl<'a> IntoIterator for &'a SymbolAbsPath {
    type Item = &'a Identifier;
    type IntoIter = std::slice::Iter<'a, Identifier>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// 2. Enables: for part in abs_path { ... } (takes ownership)
impl IntoIterator for SymbolAbsPath {
    type Item = Identifier;
    type IntoIter = std::vec::IntoIter<Identifier>;

    fn into_iter(self) -> Self::IntoIter {
        self.parts.into_iter()
    }
}

// 3. Enables indexing: abs_path[0]
impl std::ops::Index<usize> for SymbolAbsPath {
    type Output = Identifier;

    fn index(&self, index: usize) -> &Self::Output {
        &self.parts[index]
    }
}
