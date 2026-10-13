// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod manifest;

mod display;

pub mod symbol;
pub mod symbol_path;
pub mod visitor;

use std::collections::BTreeMap;

pub use manifest::{Dependency, LibrarySection, Manifest, ManifestError};
use microcad_lang_base::{ExternalId, HashSet, LibraryId, LibraryInfo, Name, Version};

use microcad_lang_lower::ir::UnresolvedPath;
use microcad_macros::Artifact;
use serde::{Deserialize, Serialize};

pub use symbol::*;

/// The leading path part, which refers to the parent module, e.g. `super::geo::Circle`.
const SUPER: &str = "super";

/// Looks up path parts inside a module of another library.
pub type ExternalLookUp<'a> = dyn Fn(&ExternalId, &[Identifier]) -> Option<SymbolId> + 'a;

/// A symbol found by a look up.
#[derive(Debug, Clone, PartialEq)]
pub enum LookUp {
    /// A symbol of the library itself.
    Local(SymbolNodeId),
    /// A symbol of another library, imported e.g. by `use std::geo2d::*;`.
    External(SymbolId),
}

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
    ///
    /// Symbols imported from other libraries are not found, see [`Library::look_up_symbol`].
    pub fn look_up(
        &self,
        starting_node: SymbolNodeId,
        path: &UnresolvedPath,
    ) -> Option<SymbolNodeId> {
        match self.look_up_symbol(starting_node, path, &|_, _| None)? {
            LookUp::Local(id) => Some(id),
            LookUp::External(_) => None,
        }
    }

    /// Look up a symbol, which may be imported from another library, e.g. by `use std::geo2d::*;`.
    ///
    /// `external` looks up path parts inside a module of another library.
    pub fn look_up_symbol(
        &self,
        starting_node: SymbolNodeId,
        path: &UnresolvedPath,
        external: &ExternalLookUp,
    ) -> Option<LookUp> {
        let mut visited_wildcards = HashSet::default();
        self.look_up_internal(starting_node, path, external, &mut visited_wildcards)
    }

    fn look_up_internal(
        &self,
        starting_node: SymbolNodeId,
        path: &UnresolvedPath,
        external: &ExternalLookUp,
        visited_wildcards: &mut HashSet<SymbolNodeId>,
    ) -> Option<LookUp> {
        // 0. Path starting with `super::`: start from the parent of the current module.
        if path.parts.first().is_some_and(|part| part == SUPER) {
            if path.is_absolute {
                return None;
            }
            let mut module = self.module_of(starting_node);
            let mut parts = &path.parts[..];
            while let Some((part, rest)) = parts.split_first()
                && part == SUPER
            {
                module = self.module_of(module.parent(&self.arena)?);
                parts = rest;
            }
            return self.look_up_downwards(
                module,
                parts,
                starting_node,
                external,
                visited_wildcards,
            );
        }

        // 1. Absolute Path: start directly from library root
        if path.is_absolute {
            return self.look_up_downwards(
                self.root,
                &path.parts,
                starting_node,
                external,
                visited_wildcards,
            );
        }

        // 2. Relative Path: Bubble UP through parent lexical scopes
        let mut curr_scope = Some(starting_node);
        while let Some(scope_id) = curr_scope {
            match self.find_child_or_wildcard(
                scope_id,
                &path.parts,
                starting_node,
                external,
                visited_wildcards,
            )? {
                Some(LookUp::Local(matched_node)) => {
                    return self.look_up_downwards(
                        matched_node,
                        &path.parts[1..],
                        starting_node,
                        external,
                        visited_wildcards,
                    );
                }
                // An external symbol has been looked up with all parts.
                Some(found) => return Some(found),
                None => (),
            }

            // Move up to parent scope
            curr_scope = scope_id.parent(&self.arena);
        }

        None
    }

    /// Search a scope for a child named like the first part of `parts`,
    /// falling back to wildcard imports in that scope.
    ///
    /// Wildcards are visible like other symbols, so a module imported by a wildcard
    /// re-exports only the symbols of its public wildcards (`pub use a::*;`).
    ///
    /// Returns `Some(None)` if nothing has been found, `None` if `parts` is empty.
    /// A symbol found in another library is looked up with all `parts` already.
    fn find_child_or_wildcard(
        &self,
        scope_id: SymbolNodeId,
        parts: &[Identifier],
        from_scope: SymbolNodeId,
        external: &ExternalLookUp,
        visited_wildcards: &mut HashSet<SymbolNodeId>,
    ) -> Option<Option<LookUp>> {
        let name = parts.first()?;

        // Step A: Check direct named child
        let scope = SymbolNodeRef::new(scope_id, &self.arena);
        if let Some(child) = scope.find_symbol_node(name)
            && self.is_visible(child.id, from_scope)
        {
            return Some(Some(LookUp::Local(child.id)));
        }

        // Step B: Search inside wildcard imports declared in `scope_id`
        for child in scope.children() {
            let SymbolDef::Wildcard(wildcard) = &child.get().def else {
                continue;
            };
            if !self.is_visible(child.id, from_scope) {
                continue;
            }

            // Prevent cyclic wildcard loops
            if !visited_wildcards.insert(child.id) {
                continue;
            }

            // The module the wildcard `a::path::*` imports from.
            let target_module_id = match &wildcard.path {
                // Not resolved yet, e.g. while resolving the wildcard itself.
                Path::Unresolved(unresolved_path) => match self.look_up_internal(
                    scope_id,
                    unresolved_path,
                    external,
                    visited_wildcards,
                ) {
                    Some(LookUp::Local(id)) => id,
                    _ => continue,
                },
                Path::Resolved(SymbolId::Item(id)) => *id,
                Path::Resolved(SymbolId::External(module)) => match external(module, parts) {
                    Some(symbol_id) => return Some(Some(LookUp::External(symbol_id))),
                    None => continue,
                },
                _ => continue,
            };

            // Search for `name` inside the target module, including its re-exports.
            if let Some(found) = self.find_child_or_wildcard(
                target_module_id,
                parts,
                from_scope,
                external,
                visited_wildcards,
            )? {
                return Some(Some(found));
            }
        }

        Some(None)
    }

    /// Traverse downwards through child symbols, evaluating wildcards at each step.
    fn look_up_downwards(
        &self,
        start: SymbolNodeId,
        parts: &[Identifier],
        from_scope: SymbolNodeId,
        external: &ExternalLookUp,
        visited_wildcards: &mut HashSet<SymbolNodeId>,
    ) -> Option<LookUp> {
        let mut curr = start;
        for (i, _) in parts.iter().enumerate() {
            match self.find_child_or_wildcard(
                curr,
                &parts[i..],
                from_scope,
                external,
                visited_wildcards,
            )?? {
                LookUp::Local(id) => curr = id,
                // An external symbol has been looked up with the remaining parts.
                found => return Some(found),
            }
        }
        Some(LookUp::Local(curr))
    }

    /// Look up public path parts inside a module of this library, e.g. for a wildcard import.
    ///
    /// `external` looks up symbols, which the module imports from other libraries.
    pub fn look_up_in_module(
        &self,
        module: SymbolNodeId,
        parts: &[Identifier],
        external: &ExternalLookUp,
    ) -> Option<LookUp> {
        // From the root, only public symbols inside the module are visible.
        self.look_up_downwards(module, parts, self.root, external, &mut HashSet::default())
    }

    /// The module (root, inline module or source file) which contains `node` or is `node`.
    fn module_of(&self, mut node: SymbolNodeId) -> SymbolNodeId {
        loop {
            let def = &SymbolNodeRef::new(node, &self.arena).get().def;
            if matches!(
                def,
                SymbolDef::Root(_) | SymbolDef::InlineModule(_) | SymbolDef::SourceFile(_)
            ) {
                return node;
            }
            match node.parent(&self.arena) {
                Some(parent) => node = parent,
                None => return node,
            }
        }
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
