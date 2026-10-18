// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Search for library symbols.

use mucad_lang_base::{HashSet, Identifier, SymbolId};
use mucad_lang_lower::ir::{Path, UnresolvedPath};

use crate::{
    ExternalLookUp, Library, LookUp, SymbolDef, SymbolNodeExt, SymbolNodeRef, library::SymbolNodeId,
};

/// The leading path part, which refers to the parent module, e.g. `super::geo::Circle`.
const SUPER: &str = "super";

/// A symbol look up in a library.
pub struct Search<'a> {
    lib: &'a Library,
    /// The scope the look up starts from, which determines the visible symbols.
    from_scope: SymbolNodeId,
    /// Looks up path parts inside a module of another library.
    external: &'a ExternalLookUp<'a>,
    /// Wildcards and aliases searched already, to stop at cyclic imports.
    visited: HashSet<SymbolNodeId>,
}

impl<'a> Search<'a> {
    pub fn new(
        lib: &'a Library,
        from_scope: SymbolNodeId,
        external: &'a ExternalLookUp<'a>,
    ) -> Self {
        Self {
            lib,
            from_scope,
            external,
            visited: HashSet::default(),
        }
    }

    /// Look up a path from `from_scope`.
    pub fn path(&mut self, path: &UnresolvedPath) -> Option<LookUp> {
        let lib = self.lib;
        let parts = &path.parts[..];
        let supers = parts.iter().take_while(|part| *part == SUPER).count();

        match (path.is_absolute, supers) {
            // An absolute path starts from the library root, `::super` does not exist.
            (true, 0) => self.downwards(lib.root, parts),
            (true, _) => None,
            // A relative path starts with a symbol in the scope or one of its parent scopes.
            (false, 0) => {
                let found =
                    std::iter::successors(Some(self.from_scope), |scope| scope.parent(&lib.arena))
                        .find_map(|scope| self.in_scope(scope, parts))?;
                self.descend(found, &parts[1..])
            }
            // `super::` parts start from the parent of the current module.
            (false, _) => {
                let module = (0..supers)
                    .try_fold(lib.module_of(self.from_scope), |module, _| {
                        Some(lib.module_of(module.parent(&lib.arena)?))
                    })?;
                self.downwards(module, &parts[supers..])
            }
        }
    }

    /// Look up `parts` one by one, starting inside `start`.
    pub fn downwards(&mut self, start: SymbolNodeId, parts: &[Identifier]) -> Option<LookUp> {
        match parts {
            [] => Some(LookUp::Local(start)),
            [_, rest @ ..] => {
                let found = self.in_scope(start, parts)?;
                self.descend(found, rest)
            }
        }
    }

    /// Look up the `rest` of a path inside a `found` symbol.
    pub fn descend(&mut self, found: LookUp, rest: &[Identifier]) -> Option<LookUp> {
        match found {
            LookUp::Local(id) if rest.is_empty() => Some(LookUp::Local(id)),
            LookUp::Local(id) => match &SymbolNodeRef::new(id, &self.lib.arena).get().def {
                SymbolDef::Alias(alias) => self.in_alias(id, &alias.path, rest),
                _ => self.downwards(id, rest),
            },
            // A symbol of another library has been looked up with all parts.
            LookUp::External(_) => Some(found),
        }
    }

    /// Look up the `rest` of a path inside the module the alias `node` stands for,
    /// e.g. `space::Sphere` with `use std::geo3d as space;`.
    fn in_alias(&mut self, node: SymbolNodeId, path: &Path, rest: &[Identifier]) -> Option<LookUp> {
        // Each alias is followed once, to stop at cyclic imports.
        if !self.visited.insert(node) {
            return None;
        }

        let target = match path {
            Path::Resolved(SymbolId::Item(module)) => LookUp::Local(*module),
            Path::Resolved(module @ SymbolId::External(_)) => LookUp::External(module.clone()),
            // Not resolved yet, e.g. while resolving other imports.
            Path::Unresolved(path) => {
                let scope = node.parent(&self.lib.arena)?;
                self.with_scope(scope, |search| search.path(path))?
            }
            _ => return None,
        };
        match target {
            LookUp::Local(module) => self.descend(LookUp::Local(module), rest),
            LookUp::External(SymbolId::External(module)) => {
                (self.external)(&module, rest).map(LookUp::External)
            }
            LookUp::External(_) => None,
        }
    }

    /// Look up the first of `parts` in `scope`: a child or a symbol imported by a wildcard.
    pub fn in_scope(&mut self, scope: SymbolNodeId, parts: &[Identifier]) -> Option<LookUp> {
        let name = parts.first()?;
        let scope = SymbolNodeRef::new(scope, &self.lib.arena);

        // Children shadow imported symbols.
        scope
            .find_symbol_node(name)
            .filter(|child| self.lib.is_visible(child.id, self.from_scope))
            .map(|child| LookUp::Local(child.id))
            .or_else(|| {
                scope
                    .children()
                    .find_map(|child| self.in_wildcard(scope.id, child, parts))
            })
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
        // Each wildcard is searched once, to stop at cyclic imports.
        if !self.lib.is_visible(node.id, self.from_scope) || !self.visited.insert(node.id) {
            return None;
        }

        match &wildcard.path {
            Path::Resolved(SymbolId::Item(module)) => self.in_scope(*module, parts),
            Path::Resolved(SymbolId::External(module)) => {
                (self.external)(module, parts).map(LookUp::External)
            }
            // Not resolved yet, e.g. while resolving the wildcard itself.
            Path::Unresolved(path) => match self.with_scope(scope, |search| search.path(path))? {
                LookUp::Local(module) => self.in_scope(module, parts),
                LookUp::External(_) => None,
            },
            _ => None,
        }
    }

    /// Run `f` with a look up from `scope` instead of `from_scope`.
    fn with_scope<T>(&mut self, scope: SymbolNodeId, f: impl FnOnce(&mut Self) -> T) -> T {
        let from_scope = std::mem::replace(&mut self.from_scope, scope);
        let result = f(self);
        self.from_scope = from_scope;
        result
    }
}
