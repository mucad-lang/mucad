// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod manifest;

mod display;

pub mod symbol;
pub mod symbol_path;
pub mod visitor;

use std::collections::BTreeMap;

pub use manifest::{Dependency, LibrarySection, Manifest, ManifestError};
use microcad_lang_base::{HashSet, LibraryId, LibraryInfo, Name, Version};

use microcad_lang_lower::ir::UnresolvedPath;
use microcad_macros::Artifact;
use serde::{Deserialize, Serialize};

pub use symbol::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Artifact)]
pub struct Library {
    pub manifest: Option<Manifest>,

    /// Symbol node id of `mu` symbol that contains all external dependencies
    pub dependencies: BTreeMap<Name, LibraryId>,

    /// The symbol root.
    pub root: SymbolNodeId,

    pub arena: SymbolArena,
}

impl std::hash::Hash for Library {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root().descendants().for_each(|node| node.hash(state));
    }
}

impl Library {
    /// An empty lib without dependencies or source file.
    pub fn new() -> Self {
        let mut arena = SymbolArena::default();
        let root = arena.new_node(Symbol::root(None));
        Self {
            manifest: None,
            dependencies: Default::default(),
            root,
            arena,
        }
    }

    pub fn with_manifest(&mut self, manifest: Manifest) -> &mut Self {
        self.manifest = Some(manifest);
        self
    }

    /// Returns true if this library does not have standard library as dependency.
    pub fn no_std(&self) -> bool {
        match &self.manifest {
            Some(manifest) => manifest.library.no_std.unwrap_or(false),
            None => false, // Load std by default
        }
    }

    pub fn name(&self) -> Option<&Name> {
        self.manifest
            .as_ref()
            .map(|manifest| &manifest.library.name)
    }

    pub fn version(&self) -> Option<&Version> {
        self.manifest
            .as_ref()
            .map(|manifest| &manifest.library.version)
    }

    pub fn info(&self) -> Option<LibraryInfo> {
        match (self.name(), self.version()) {
            (Some(name), Some(ver)) => Some(LibraryInfo::new(name.clone(), ver.clone())),
            _ => None,
        }
    }

    pub fn id(&self) -> LibraryId {
        self.info()
            .map(|info| info.into())
            .unwrap_or(LibraryId::ROOT)
    }

    pub fn root<'a>(&'a self) -> SymbolNodeRef<'a> {
        SymbolNodeRef::new(self.root, &self.arena)
    }

    pub fn root_mut<'a>(&'a mut self) -> SymbolNodeMut<'a> {
        SymbolNodeMut::new(self.root, &mut self.arena)
    }

    pub fn append_symbol(&mut self, symbol: impl Into<Symbol>) -> SymbolNodeId {
        self.root.append_value(symbol.into(), &mut self.arena)
    }

    /// Public entry point for symbol lookup.
    pub fn look_up(
        &self,
        starting_node: SymbolNodeId,
        path: &UnresolvedPath,
    ) -> Option<SymbolNodeId> {
        let mut visited_wildcards = HashSet::default();
        self.look_up_internal(starting_node, path, &mut visited_wildcards)
    }

    fn look_up_internal(
        &self,
        starting_node: SymbolNodeId,
        path: &UnresolvedPath,
        visited_wildcards: &mut HashSet<SymbolNodeId>,
    ) -> Option<SymbolNodeId> {
        let first_part = path.parts.first()?;
        let first_name = first_part.clone();

        // 1. Absolute Path: start directly from library root
        if path.is_absolute {
            return self.look_up_downwards(
                self.root,
                &path.parts,
                starting_node,
                visited_wildcards,
            );
        }

        // 2. Relative Path: Bubble UP through parent lexical scopes
        let mut curr_scope = Some(starting_node);
        while let Some(scope_id) = curr_scope {
            if let Some(matched_node) =
                self.find_child_or_wildcard(scope_id, &first_name, starting_node, visited_wildcards)
            {
                if path.parts.len() == 1 {
                    return Some(matched_node);
                } else {
                    return self.look_up_downwards(
                        matched_node,
                        &path.parts[1..],
                        starting_node,
                        visited_wildcards,
                    );
                }
            }

            // Move up to parent scope
            curr_scope = scope_id.parent(&self.arena);
        }

        None
    }

    /// Search a scope for a child by name, falling back to wildcard imports in that scope.
    fn find_child_or_wildcard(
        &self,
        scope_id: SymbolNodeId,
        name: &Identifier,
        from_scope: SymbolNodeId,
        visited_wildcards: &mut HashSet<SymbolNodeId>,
    ) -> Option<SymbolNodeId> {
        // Step A: Check direct named child
        let scope = SymbolNodeRef::new(scope_id, &self.arena);
        if let Some(child) = scope.find_symbol_node(name)
            && self.is_visible(child.id, from_scope)
        {
            return Some(child.id);
        }

        // Step B: Search inside wildcard imports declared in `scope_id`
        for child in scope.children() {
            let symbol = child.get();

            if let SymbolDef::Wildcard(wildcard) = &symbol.def {
                // Prevent cyclic wildcard loops
                if !visited_wildcards.insert(child.id) {
                    continue;
                }

                if let Path::Unresolved(unresolved_path) = &wildcard.path {
                    // Look up the target path of the wildcard `a::path`
                    if let Some(target_module_id) =
                        self.look_up_internal(scope_id, unresolved_path, visited_wildcards)
                    {
                        let target_ref = SymbolNodeRef::new(target_module_id, &self.arena);

                        // Search for `name` directly inside the target module
                        if let Some(imported_node) = target_ref.find_symbol_node(name)
                            && self.is_visible(imported_node.id, from_scope)
                        {
                            return Some(imported_node.id);
                        }
                    }
                }
            }
        }

        None
    }

    /// Traverse downwards through child symbols, evaluating wildcards at each step.
    fn look_up_downwards(
        &self,
        start: SymbolNodeId,
        parts: &[Identifier],
        from_scope: SymbolNodeId,
        visited_wildcards: &mut HashSet<SymbolNodeId>,
    ) -> Option<SymbolNodeId> {
        let mut curr = start;
        for part in parts {
            let name = part.clone();
            curr = self.find_child_or_wildcard(curr, &name, from_scope, visited_wildcards)?;
        }
        Some(curr)
    }

    /// Evaluates if `target` symbol is accessible from `from_scope`.
    pub fn is_visible(&self, target: SymbolNodeId, from_scope: SymbolNodeId) -> bool {
        let Some(target_symbol) = self.arena.get(target) else {
            return false;
        };

        match target_symbol.get().meta.vis {
            // Public items are accessible from anywhere
            Visibility::Public => true,

            // Private items are visible if `from_scope` is inside `target`'s defining module
            Visibility::Private => {
                let target_parent = target.parent(&self.arena);

                match target_parent {
                    Some(parent) => self.is_descendant_or_same(from_scope, parent),
                    None => true, // Root-level items
                }
            }
        }
    }

    /// Checks if `node` is equal to or enclosed within `ancestor`.
    fn is_descendant_or_same(&self, mut node: SymbolNodeId, ancestor: SymbolNodeId) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            match node.parent(&self.arena) {
                Some(parent) => node = parent,
                None => return false,
            }
        }
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}
