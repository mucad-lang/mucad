// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad resolved symbol tree (RST).

use microcad_lang_base::VersionAnnotation;
use microcad_lang_lower::ir;
use strum::IntoStaticStr;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use derive_more::From;

pub use microcad_lang_base::{Identifier, SymbolId};

/// Function definitions from IR.
pub mod function {
    use microcad_lang_lower::ir;

    pub use ir::{
        Function, FunctionCall, FunctionExpression, FunctionIf, FunctionSignature,
        FunctionStatement, ReturnStatement, Scope,
    };
    pub type Argument = ir::Argument<FunctionExpression>;
    pub type ArgumentList = ir::ArgumentList<FunctionExpression>;
}

pub use function::{Function, FunctionExpression, FunctionStatement};

/// Workbench definitions from IR.
pub mod workbench {
    use microcad_lang_lower::ir;

    pub use ir::{
        Group, Init, InitStatement, Marker, ModelAttributes, Workbench, WorkbenchCall,
        WorkbenchExpression, WorkbenchIf, WorkbenchKind, WorkbenchSignature, WorkbenchStatement,
    };
    pub type Argument = ir::Argument<WorkbenchExpression>;
    pub type ArgumentList = ir::ArgumentList<WorkbenchExpression>;
}

pub use workbench::{
    ModelAttributes, Workbench, WorkbenchExpression, WorkbenchKind, WorkbenchStatement,
};

pub mod constant {
    use microcad_lang_lower::ir;

    pub use ir::{Constant, ConstantExpression, ConstantValue};

    pub type Argument = ir::Argument<ir::ConstantExpression>;
    pub type ArgumentList = ir::ArgumentList<ir::ConstantExpression>;
}

pub use ir::{
    Alias, Argument, ArgumentList, Call, Constant, ConstantExpression, ConstantValue, DocBlock,
    ExportAttribute, ExprSpec, InlineModule, Meta, Parameter, ParameterList, Path, Source,
    SourceStatement, Visibility, Wildcard,
};

use crate::library::symbol_path::SymbolAbsPath;

#[derive(Debug, Clone, From, Hash, PartialEq, Serialize, Deserialize)]
pub enum SourceFile {
    NotLoaded,
    Loaded {
        /// Relative path from project root of loaded file module
        path: std::path::PathBuf,
        /// Source.
        source: Source,
    },
    /// A source file with errors, which could not be compiled.
    Failed {
        /// Relative path from project root of the file module
        path: std::path::PathBuf,
    },
}

/// Symbol definition
#[derive(Debug, Clone, From, IntoStaticStr, Hash, PartialEq, Serialize, Deserialize)]
pub enum SymbolDef {
    Root(Option<SourceFile>),
    /// Inline Module symbol: `mod foo {}`
    InlineModule(InlineModule),
    /// Source File
    SourceFile(SourceFile),
    /// Workbench symbol.
    Workbench(Workbench),
    /// Function symbol.
    Function(Function),
    /// Constant.
    Constant(Constant),
    /// Alias of a pub use statement.
    Alias(Alias),
    /// Use all available symbols in the module with the given name.
    Wildcard(Wildcard),
}

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
        source: Source,
    ) -> Self {
        Self {
            meta: item.meta.clone(),
            doc: item.doc.clone(),
            ver: item.ver.clone(),
            def: SymbolDef::SourceFile(SourceFile::Loaded { path, source }),
        }
    }
}

impl Symbol {
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

impl From<ir::Def> for SymbolDef {
    fn from(def: ir::Def) -> Self {
        match def {
            ir::Def::InlineModule(inline_module) => SymbolDef::InlineModule(inline_module),
            ir::Def::FileModule(_) | ir::Def::Source(_) => {
                // Source files are loaded in `Library::load_source`
                SymbolDef::SourceFile(SourceFile::NotLoaded)
            }
            ir::Def::Workbench(workbench) => SymbolDef::Workbench(workbench),
            ir::Def::Function(function) => SymbolDef::Function(function),
            ir::Def::Constant(constant) => SymbolDef::Constant(constant),
            ir::Def::Alias(alias) => SymbolDef::Alias(alias),
            ir::Def::Wildcard(wildcard) => SymbolDef::Wildcard(wildcard),
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
    pub fn root(lib_mu: Option<SourceFile>) -> Self {
        Self {
            meta: Meta {
                vis: Visibility::Public,
                ..Default::default()
            },
            def: SymbolDef::Root(lib_mu),
            doc: Default::default(),
            ver: Default::default(),
        }
    }
}

pub type SymbolArena = microcad_lang_base::tree::Arena<Symbol>;
pub type SymbolNode = microcad_lang_base::tree::Node<Symbol>;
pub type SymbolNodeRef<'a> = microcad_lang_base::tree::NodeRef<'a, Symbol>;
pub type SymbolNodeMut<'a> = microcad_lang_base::tree::NodeMut<'a, Symbol>;
pub type SymbolNodeId = microcad_lang_base::tree::NodeId;

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
