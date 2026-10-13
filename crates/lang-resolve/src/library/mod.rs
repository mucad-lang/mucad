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
        Search::new(self, starting_node, external).path(path)
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
        Search::new(self, self.root, external).downwards(module, parts)
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

/// A symbol look up in a library.
struct Search<'a> {
    lib: &'a Library,
    /// The scope the look up starts from, which determines the visible symbols.
    from_scope: SymbolNodeId,
    /// Looks up path parts inside a module of another library.
    external: &'a ExternalLookUp<'a>,
    /// Wildcards searched already, to stop at cyclic wildcard imports.
    visited_wildcards: HashSet<SymbolNodeId>,
}

impl<'a> Search<'a> {
    fn new(lib: &'a Library, from_scope: SymbolNodeId, external: &'a ExternalLookUp<'a>) -> Self {
        Self {
            lib,
            from_scope,
            external,
            visited_wildcards: HashSet::default(),
        }
    }

    /// Look up a path from `from_scope`.
    fn path(&mut self, path: &UnresolvedPath) -> Option<LookUp> {
        // `super::` parts start from the parent of the current module.
        if path.parts.first().is_some_and(|part| part == SUPER) {
            if path.is_absolute {
                return None;
            }
            let mut module = self.lib.module_of(self.from_scope);
            let mut parts = &path.parts[..];
            while let Some((part, rest)) = parts.split_first()
                && part == SUPER
            {
                module = self.lib.module_of(module.parent(&self.lib.arena)?);
                parts = rest;
            }
            return self.downwards(module, parts);
        }

        // An absolute path starts from the library root.
        if path.is_absolute {
            return self.downwards(self.lib.root, &path.parts);
        }

        // A relative path starts with a symbol in the scope or one of its parent scopes.
        let mut scope = Some(self.from_scope);
        while let Some(scope_id) = scope {
            match self.in_scope(scope_id, &path.parts) {
                Some(LookUp::Local(id)) => return self.downwards(id, &path.parts[1..]),
                // An external symbol has been looked up with all parts.
                Some(found) => return Some(found),
                None => scope = scope_id.parent(&self.lib.arena),
            }
        }
        None
    }

    /// Look up `parts` one by one, starting inside `start`.
    fn downwards(&mut self, start: SymbolNodeId, parts: &[Identifier]) -> Option<LookUp> {
        let mut curr = start;
        for i in 0..parts.len() {
            match self.in_scope(curr, &parts[i..])? {
                LookUp::Local(id) => curr = id,
                // An external symbol has been looked up with the remaining parts.
                found => return Some(found),
            }
        }
        Some(LookUp::Local(curr))
    }

    /// Look up the first of `parts` in `scope`: a child or a symbol imported by a wildcard.
    fn in_scope(&mut self, scope: SymbolNodeId, parts: &[Identifier]) -> Option<LookUp> {
        let name = parts.first()?;
        let scope = SymbolNodeRef::new(scope, &self.lib.arena);

        // Children shadow imported symbols.
        if let Some(child) = scope.find_symbol_node(name)
            && self.lib.is_visible(child.id, self.from_scope)
        {
            return Some(LookUp::Local(child.id));
        }

        scope
            .children()
            .find_map(|child| self.in_wildcard(scope.id, child, parts))
    }

    /// Look up the first of `parts` in the module imported by `node`, if it is a wildcard.
    ///
    /// Wildcards are visible like other symbols, so a module imported by a wildcard
    /// re-exports only the symbols of its public wildcards (`pub use a::*;`).
    /// A symbol of another library is looked up with all `parts`.
    fn in_wildcard(
        &mut self,
        scope: SymbolNodeId,
        node: SymbolNodeRef<'a>,
        parts: &[Identifier],
    ) -> Option<LookUp> {
        let SymbolDef::Wildcard(wildcard) = &node.get().def else {
            return None;
        };
        if !self.lib.is_visible(node.id, self.from_scope) || !self.visited_wildcards.insert(node.id)
        {
            return None;
        }

        match &wildcard.path {
            Path::Resolved(SymbolId::Item(module)) => self.in_scope(*module, parts),
            Path::Resolved(SymbolId::External(module)) => {
                (self.external)(module, parts).map(LookUp::External)
            }
            // Not resolved yet, e.g. while resolving the wildcard itself.
            Path::Unresolved(path) => {
                let from_scope = std::mem::replace(&mut self.from_scope, scope);
                let module = self.path(path);
                self.from_scope = from_scope;
                match module? {
                    LookUp::Local(module) => self.in_scope(module, parts),
                    LookUp::External(_) => None,
                }
            }
            _ => None,
        }
    }
}
