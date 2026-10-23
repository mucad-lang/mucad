// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Symbols to complete a path, e.g. in an editor.

use mucad_lang_base::{
    ExternalId, Identifier, LibraryId, Shared, Source, SymbolId, element::WorkbenchKind,
};
use mucad_lang_lower::ir::UnresolvedPath;

use crate::{Library, ResolveContext, SymbolDef, library::Search};

use super::ExternalResolver;

/// The kind of a symbol to complete.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionKind {
    /// A library, e.g. `std`.
    Library,
    /// An inline module or a file module.
    Module,
    /// A sketch, part or operation.
    Workbench(WorkbenchKind),
    /// A function.
    Function,
    /// A constant.
    Constant,
    /// A built-in under the name of a library symbol, e.g. `print` for `__mu::debug::print`.
    Builtin,
}

/// A symbol to complete a path with.
#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    /// The name of the symbol.
    pub name: Identifier,
    /// The kind of the symbol.
    pub kind: CompletionKind,
    /// The documentation of the symbol, if it has some.
    pub doc: Option<String>,
}

impl ResolveContext {
    /// The symbols to complete a path at the byte `offset` of `source` in the main library,
    /// sorted by name.
    ///
    /// `qualifier` is the path before the name to complete, e.g. `std::geo2d` for `std::geo2d::Ci`.
    /// Without qualifier (no parts, not absolute), the symbols visible at `offset` are completed:
    /// of the main library, the names of its dependencies (e.g. `std`) and the prelude.
    /// Symbols with the same name have the same priority as when resolving.
    ///
    /// The scope of `offset` is determined by [`Library::scope_at`].
    /// Built-ins (`__mu::`) are not completed.
    pub fn completions(
        &self,
        source: &Source,
        offset: usize,
        qualifier: &UnresolvedPath,
    ) -> Vec<Completion> {
        if qualifier.builtin_id().is_some() {
            return Vec::new();
        }

        // The cache is locked for each access only, the external search locks it, too.
        let library = |id: &LibraryId| self.lib_cache.read_unwrap().get(id);
        let main = library(&LibraryId::ROOT);
        let dependencies = main.as_ref().map_or_else(
            || self.implicit_dependencies.clone(),
            |lib| lib.dependencies.clone(),
        );
        let external = ExternalResolver::new(self.lib_cache.clone());
        let is_name = qualifier.parts.is_empty() && !qualifier.is_absolute;

        // In the order of their priority, a symbol shadows later ones with the same name.
        let mut symbols: Vec<(Identifier, Option<SymbolId>)> = Vec::new();

        if let Some(lib) = &main {
            let scope = lib.scope_at(source, offset);
            let mut search = Search::new(lib, scope, &external);
            let found = match (is_name, qualifier.parts.is_empty()) {
                (true, _) => search.visible_symbols(),
                (false, true) => search.symbols_in(lib.root),
                (false, false) => search.symbols_in_path(qualifier),
            };
            symbols.extend(
                found
                    .into_iter()
                    .map(|(name, found)| (name, Some(found.symbol_id(lib, None)))),
            );
        }

        if is_name {
            // Libraries are completed without symbol.
            symbols.extend(dependencies.keys().map(|name| (name.clone().into(), None)));
            if let Some(prelude) = &self.prelude
                && dependencies
                    .values()
                    .any(|lib_id| *lib_id == prelude.library)
            {
                symbols.extend(
                    prelude
                        .symbols
                        .iter()
                        .map(|(name, symbol_id)| (name.clone(), Some(symbol_id.clone()))),
                );
            }
        } else if let Some(name) = qualifier.external_name()
            && let Some(lib) = dependencies.get(&name).and_then(library)
        {
            // The library appears in its own tree under its name, so only `mu` is removed.
            let path = qualifier.without_mu_prefix();
            symbols.extend(
                Search::new(&lib, lib.root, &external)
                    .symbols_in_path(&path)
                    .into_iter()
                    .map(|(name, found)| (name, Some(found.symbol_id(&lib, Some(lib.id()))))),
            );
        }

        let mut completions: Vec<Completion> = Vec::new();
        for (name, symbol_id) in symbols {
            if completions.iter().any(|completion| completion.name == name) {
                continue;
            }
            let completion = match symbol_id {
                Some(symbol_id) => Completion::new(name, &symbol_id, main.as_ref(), library),
                None => Some(Completion::library(name)),
            };
            completions.extend(completion);
        }
        completions.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
        completions
    }
}

impl Completion {
    /// The completion of the symbol `symbol_id` with `name`.
    ///
    /// `main` is the library of [`SymbolId::Item`], `lib` gets the library of a
    /// [`SymbolId::External`]. Wildcards, unresolved aliases and locals are not completed.
    pub(crate) fn new(
        name: Identifier,
        symbol_id: &SymbolId,
        main: Option<&Shared<Library>>,
        lib: impl Fn(&LibraryId) -> Option<Shared<Library>>,
    ) -> Option<Self> {
        let (lib, node) = match symbol_id {
            SymbolId::Item(node) => (main?.clone(), *node),
            SymbolId::External(ExternalId { lib_id, id }) => (lib(lib_id)?, *id),
            SymbolId::Builtin(_) => {
                return Some(Self {
                    name,
                    kind: CompletionKind::Builtin,
                    doc: None,
                });
            }
            SymbolId::Local(_) => return None,
        };

        let symbol = lib.arena.get(node)?.get();
        let kind = match &symbol.def {
            SymbolDef::Root | SymbolDef::InlineModule(_) | SymbolDef::SourceFile(_) => {
                CompletionKind::Module
            }
            SymbolDef::Workbench(workbench) => CompletionKind::Workbench(workbench.signature.kind),
            SymbolDef::Function(_) => CompletionKind::Function,
            SymbolDef::Constant(_) => CompletionKind::Constant,
            SymbolDef::Alias(_) | SymbolDef::Wildcard(_) => return None,
        };
        let doc = symbol.doc.content.trim();
        Some(Self {
            name,
            kind,
            doc: (!doc.is_empty()).then(|| doc.to_string()),
        })
    }

    /// The completion of a library with `name`, e.g. `std`.
    pub(crate) fn library(name: Identifier) -> Self {
        Self {
            name,
            kind: CompletionKind::Library,
            doc: None,
        }
    }
}
