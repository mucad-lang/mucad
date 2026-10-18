// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::BuiltinLibrary;
use mucad_lang_resolve::Library;

#[derive(Debug)]
pub struct BuiltinLibraryDocGenerator<'lib> {
    /// Registered builtins,
    pub lib: &'lib BuiltinLibrary,
}

impl<'lib> BuiltinLibraryDocGenerator<'lib> {
    pub fn new(lib: &'lib BuiltinLibrary) -> Self {
        Self { lib }
    }
}

/// Documentation generator for a `Library`.
pub struct LibraryDocGenerator<'lib> {
    pub lib: &'lib Library,
}

impl<'lib> LibraryDocGenerator<'lib> {
    pub fn new(lib: &'lib Library) -> Self {
        Self { lib }
    }
}
