// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Identifier, SrcRef, SrcReferrer};

use crate::library::SymbolNodeId;

/// A map of locals.
///
/// The `Vec<SrcRef>` represents the usages of this local.
#[derive(Debug, Default)]
pub struct LocalTable(mucad_lang_base::HashMap<Identifier, Vec<SrcRef>>);

impl LocalTable {
    pub fn exists(&self, id: &Identifier) -> bool {
        self.0.contains_key(id)
    }
}

pub trait ScopeAccess {
    /// Get current local table, if any.
    fn local_table(&self) -> Option<&LocalTable>;
    fn local_table_mut(&mut self) -> Option<&mut LocalTable>;

    fn declare_local(&mut self, id: Identifier) {
        if let Some(local_table) = self.local_table_mut() {
            local_table.0.insert(id, vec![]);
        }
    }
    /// Returns true if the local exists.
    fn use_local(&mut self, id: &Identifier) -> bool {
        if let Some(local_table) = self.local_table_mut()
            && let Some(local) = local_table.0.get_mut(id)
        {
            local.push(id.src_ref());
            true
        } else {
            false
        }
    }

    fn unused_locals(&self) -> impl Iterator<Item = &Identifier> {
        self.local_table()
            .unwrap()
            .0
            .iter()
            .filter(|(_, usages)| usages.is_empty())
            .map(|(id, _)| id)
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
