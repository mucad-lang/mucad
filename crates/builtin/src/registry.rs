// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad built-in registry.

use microcad_lang_base::{BuiltinId, HashMap, LookUpName, ToCompactString};

use crate::{BuiltinItem, BuiltinModule};

#[derive(Default)]
pub struct BuiltinRegistry {
    builtins: HashMap<BuiltinId, &'static BuiltinItem>,
    modules: Vec<&'static BuiltinModule>,
}

impl std::fmt::Debug for BuiltinRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltinRegistry")
    }
}

impl BuiltinRegistry {
    /// Create a new [`BuiltinRegistry`] and register default modules.
    pub fn new() -> Self {
        let mut registry = Self::default();

        use crate::mu;
        [
            &mu::CORE,
            &mu::DEBUG,
            &mu::MATH,
            &mu::LIST,
            &mu::STRING,
            &mu::COLOR,
            &mu::GEO2D,
            &mu::OPS,
        ]
        .iter()
        .for_each(|module| registry.register(module));

        registry
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

impl LookUpName for BuiltinRegistry {
    fn look_up_built_in_name(&self, id: &BuiltinId) -> Option<microcad_lang_base::Name> {
        self.get(*id)
            .map(|builtin| builtin.name().to_compact_string())
    }
}
