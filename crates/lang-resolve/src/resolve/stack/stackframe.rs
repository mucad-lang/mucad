// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Identifier, SrcRef, SrcReferrer};

use crate::library::SymbolNodeId;

/// A local and its usages.
#[derive(Debug)]
struct Local {
    /// The declaration of the local.
    id: Identifier,
    /// The usages of the local.
    usages: Vec<SrcRef>,
    /// If the local is reported when it is unused, e.g. not for properties of a model.
    report_unused: bool,
}

impl Local {
    /// Returns if the local is reported as unused.
    ///
    /// Locals with names starting with `_` are not reported, e.g. `_x`.
    fn is_unused(&self) -> bool {
        self.report_unused && self.usages.is_empty() && !self.id.as_str().starts_with('_')
    }
}

/// A map of locals.
#[derive(Debug, Default)]
pub struct LocalTable(mucad_lang_base::HashMap<Identifier, Local>);

impl LocalTable {
    pub fn exists(&self, id: &Identifier) -> bool {
        self.0.contains_key(id)
    }

    /// Declare the local `id`, which is reported, if it is unused and `report_unused`.
    ///
    /// Returns the local declared before with the same name, if it is unused,
    /// e.g. the first `x` in `x = 1; x = 2;`.
    pub fn declare(&mut self, id: Identifier, report_unused: bool) -> Option<Identifier> {
        let replaced = self.0.remove(&id);
        self.0.insert(
            id.clone(),
            Local {
                id,
                usages: Vec::new(),
                report_unused,
            },
        );
        replaced.filter(Local::is_unused).map(|local| local.id)
    }

    /// The unused locals, sorted by their position in the source code.
    pub fn unused(&self) -> Vec<&Identifier> {
        let mut unused: Vec<_> = self
            .0
            .values()
            .filter(|local| local.is_unused())
            .map(|local| &local.id)
            .collect();
        unused.sort_by_key(|id| id.src_ref().start);
        unused
    }
}

pub trait ScopeAccess {
    /// Get current local table, if any.
    fn local_table(&self) -> Option<&LocalTable>;
    fn local_table_mut(&mut self) -> Option<&mut LocalTable>;

    fn declare_local(&mut self, id: Identifier) {
        if let Some(local_table) = self.local_table_mut() {
            local_table.declare(id, true);
        }
    }
    /// Returns true if the local exists.
    fn use_local(&mut self, id: &Identifier) -> bool {
        if let Some(local_table) = self.local_table_mut()
            && let Some(local) = local_table.0.get_mut(id)
        {
            local.usages.push(id.src_ref());
            true
        } else {
            false
        }
    }

    /// The unused locals, see [`LocalTable::unused`].
    fn unused_locals(&self) -> Vec<&Identifier> {
        self.local_table()
            .map(LocalTable::unused)
            .unwrap_or_default()
    }

    fn symbol_node_id(&self) -> Option<SymbolNodeId> {
        None
    }
}

impl ScopeAccess for LocalTable {
    fn local_table(&self) -> Option<&LocalTable> {
        Some(self)
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        Some(self)
    }
}
