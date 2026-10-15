// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod manifest;

mod display;

pub mod search;
pub mod symbol;
pub mod symbol_path;
pub mod visitor;

use std::collections::BTreeMap;

pub use manifest::{Dependency, LibrarySection, Manifest, ManifestError};
use microcad_lang_base::{ExternalId, LibraryId, LibraryInfo, Name, Version};
use microcad_lang_lower::ir::UnresolvedPath;
use microcad_macros::Artifact;
pub use search::Search;
use serde::{Deserialize, Serialize};

pub use symbol::*;
use visitor::Visitor;

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

    /// The paths of the source files loaded into this library, e.g. `lib.mu` and its file modules.
    ///
    /// Source files with errors are included, e.g. to watch them for changes.
    pub fn source_files(&self) -> Vec<std::path::PathBuf> {
        self.source_files_of(self.root)
    }

    /// The paths of the source files in the subtree of `node`, including `node` itself.
    pub fn source_files_of(&self, node: SymbolNodeId) -> Vec<std::path::PathBuf> {
        let mut files = SourceFiles::default();
        node.descendants(&self.arena)
            .for_each(|id| files.visit_node(SymbolNodeRef::new(id, &self.arena)));
        files.0
    }

    /// The paths of the source files, which the source file at `path` loads as file modules.
    ///
    /// The path must be given as loaded, the result is empty if there is no such source file.
    pub fn source_file_dependencies(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Vec<std::path::PathBuf> {
        let path = path.as_ref();
        let is_source_file = |id: &SymbolNodeId| {
            let mut files = SourceFiles::default();
            files.visit_node(SymbolNodeRef::new(*id, &self.arena));
            files.0.iter().any(|file| file == path)
        };

        match self.root.descendants(&self.arena).find(is_source_file) {
            Some(id) => self
                .source_files_of(id)
                .into_iter()
                .filter(|file| file != path)
                .collect(),
            None => Vec::new(),
        }
    }

    /// The symbol a node of this library stands for.
    ///
    /// An alias (`pub use __mu::debug::assert;`) stands for its target, aliases of aliases are followed.
    /// Symbols of an external library (`lib_id`) become [`SymbolId::External`].
    pub fn symbol_id(&self, node_id: SymbolNodeId, lib_id: Option<LibraryId>) -> SymbolId {
        let in_lib = |id| match lib_id {
            Some(lib_id) => SymbolId::External(ExternalId { lib_id, id }),
            None => SymbolId::Item(id),
        };

        // Limit the depth to stop at cyclic aliases.
        let mut node_id = node_id;
        for _ in 0..16 {
            let SymbolDef::Alias(alias) = &SymbolNodeRef::new(node_id, &self.arena).get().def
            else {
                break;
            };
            match alias.path.symbol_id() {
                Some(SymbolId::Item(target)) => node_id = *target,
                Some(target) => return target.clone(),
                // Alias not resolved yet.
                None => break,
            }
        }
        in_lib(node_id)
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

/// Collects the paths of source files.
#[derive(Default)]
struct SourceFiles(Vec<std::path::PathBuf>);

impl visitor::LeafVisitor for SourceFiles {}
impl visitor::ConstantVisitor for SourceFiles {}
impl visitor::WorkbenchExpressionVisitor for SourceFiles {}
impl visitor::WorkbenchVisitor for SourceFiles {}
impl visitor::FnVisitor for SourceFiles {}
impl visitor::SourceVisitor for SourceFiles {}

impl visitor::Visitor for SourceFiles {
    fn visit_file_path(&mut self, path: &std::path::Path) {
        self.0.push(path.to_path_buf());
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}
