// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad resolved symbol tree (RST).

pub mod def;
mod symbol_path;

use mucad_lang_base::VersionAnnotation;
use mucad_lang_lower::ir;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use derive_more::From;

pub use mucad_lang_base::{Identifier, SymbolId};

pub use def::*;
pub use symbol_path::SymbolAbsPath;

/// An Ir Item holds a definition and meta data
#[derive(Debug, Clone, Hash, From, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    /// Item metadata
    pub meta: Meta,
    /// Item definition
    pub def: SymbolDef,
    /// Item documentation
    pub doc: DocBlock,
    /// Item version annotation
    pub ver: VersionAnnotation,
}

impl Symbol {
    pub(crate) fn loaded_source_file(
        item: &ir::Item,
        path: std::path::PathBuf,
        source: ir::Source,
    ) -> Self {
        Self {
            meta: item.meta.clone(),
            doc: item.doc.clone(),
            ver: item.ver.clone(),
            def: SymbolDef::SourceFile(SourceFile::Loaded { path, source }),
        }
    }

    /// A file module with errors, declared by `item` (e.g. `mod geo;`).
    pub(crate) fn failed_source_file(item: &ir::Item, path: std::path::PathBuf) -> Self {
        Self {
            meta: item.meta.clone(),
            doc: item.doc.clone(),
            ver: item.ver.clone(),
            def: SymbolDef::SourceFile(SourceFile::Failed { path }),
        }
    }
}

impl From<ir::Item> for Symbol {
    fn from(item: ir::Item) -> Self {
        Self {
            meta: item.meta,
            def: item.def.into(),
            doc: item.doc,
            ver: item.ver,
        }
    }
}

impl Symbol {
    /// The unnamed root of a library.
    pub fn root() -> Self {
        Self {
            meta: Meta {
                vis: Visibility::Public,
                ..Default::default()
            },
            def: SymbolDef::Root,
            doc: Default::default(),
            ver: Default::default(),
        }
    }
}

pub type SymbolArena = mucad_lang_base::tree::Arena<Symbol>;
pub type SymbolNode = mucad_lang_base::tree::Node<Symbol>;
pub type SymbolNodeRef<'a> = mucad_lang_base::tree::NodeRef<'a, Symbol>;
pub type SymbolNodeMut<'a> = mucad_lang_base::tree::NodeMut<'a, Symbol>;
pub type SymbolNodeId = mucad_lang_base::tree::NodeId;

/// Extension trait for [`SymbolNode`] .
pub trait SymbolNodeExt {
    fn name(&self) -> Option<&Identifier>;

    fn is_public(&self) -> bool;

    /// Absolute path of this symbol, starting with the package root symbol.
    fn abs_path(&self) -> SymbolAbsPath;

    fn def(&self) -> &SymbolDef;

    /// Get documentation string for this symbol.
    /// The string only contains
    fn doc(&self) -> Option<&String>;

    fn find_symbol_node(&self, name: impl AsRef<str>) -> Option<SymbolNodeRef<'_>>;
}

impl<'a> SymbolNodeExt for SymbolNodeRef<'a> {
    fn name(&self) -> Option<&Identifier> {
        self.meta.name.as_ref()
    }

    fn is_public(&self) -> bool {
        self.meta.vis == Visibility::Public
    }

    fn abs_path(&self) -> SymbolAbsPath {
        let mut parts: Vec<_> = self
            .ancestors()
            .filter_map(|symbol| symbol.name().cloned())
            .collect();

        // Ancestors walk leaf -> root; reverse to get root -> leaf path
        parts.reverse();

        SymbolAbsPath { parts }
    }

    fn def(&self) -> &SymbolDef {
        &self.get().def
    }

    fn doc(&self) -> Option<&String> {
        Some(&self.get().doc.content)
    }

    fn find_symbol_node(&self, name: impl AsRef<str>) -> Option<SymbolNodeRef<'_>> {
        let name = Identifier::from(name.as_ref());
        self.children()
            .find(|child| matches!(child.name(), Some(child_name) if child_name == &name))
    }
}
