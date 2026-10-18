// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad built-in registry.

use mucad_lang_base::{BuiltinId, HashMap, LookUpName, ToCompactString};

use crate::{BuiltinItem, BuiltinModule};

/// µcad built-in library struct.
#[derive(Default)]
pub struct BuiltinLibrary {
    builtins: HashMap<BuiltinId, &'static BuiltinItem>,
    modules: Vec<&'static BuiltinModule>,
}

impl std::fmt::Debug for BuiltinLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltinRegistry")
    }
}

impl BuiltinLibrary {
    /// Create a new [`BuiltinLibrary`] with default modules.
    pub fn new() -> Self {
        let mut registry = Self::empty();

        use crate::mu;
        [
            &mu::CORE,
            &mu::DEBUG,
            &mu::MATH,
            &mu::LIST,
            &mu::STRING,
            &mu::COLOR,
            &mu::GEO2D,
            &mu::GEO3D,
            &mu::OPS,
        ]
        .iter()
        .for_each(|module| registry.register(module));

        registry
    }

    /// Create an empty [`BuiltinRegistry`] with no registered modules.
    pub fn empty() -> Self {
        Self {
            builtins: HashMap::default(),
            modules: Vec::new(),
        }
    }

    pub fn register(&mut self, builtin: &'static BuiltinItem) {
        let id = builtin.id();
        let previous = self.builtins.insert(id, builtin);

        // TODO Return a Result::Err here when we want to be able to load built-ins dynamically
        assert!(
            previous.is_none(),
            "Compiler bug: Duplicate builtin ID registered: {:?}",
            id
        );

        if let BuiltinItem::Module(builtin_module) = builtin {
            builtin_module
                .items
                .iter()
                .for_each(|item| self.register(item));
            self.modules.push(builtin_module);
        }
    }

    pub fn get(&self, id: BuiltinId) -> Option<&'static BuiltinItem> {
        self.builtins.get(&id).copied()
    }

    pub fn modules(&self) -> impl Iterator<Item = &&'static BuiltinModule> {
        self.modules.iter()
    }
}

impl LookUpName for BuiltinLibrary {
    fn look_up_built_in_name(&self, id: &BuiltinId) -> Option<mucad_lang_base::Name> {
        self.get(*id)
            .map(|builtin| builtin.name().to_compact_string())
    }
}
