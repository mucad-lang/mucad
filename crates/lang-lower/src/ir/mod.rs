// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad intermediate representation (IR) elements.
//!
//! Every AST element in the µcad language is parsed into an intermediate representation in this module.

pub mod assignment;
pub mod attribute;
pub mod cast_into;
pub mod display;
pub mod expression;
pub mod function;
pub mod parameter;
pub mod path;
pub mod source;
pub mod visitor;
pub mod workbench;

pub(crate) mod desugared;

pub use assignment::*;
pub use attribute::*;
pub use cast_into::*;
pub use expression::*;
pub use function::*;
use microcad_lang_types::Value;
pub use parameter::*;
pub use path::{Path, UnresolvedPath};
use strum::IntoStaticStr;
pub use workbench::*;

pub use source::{ExportAttribute, Source, SourceStatement};

pub use microcad_lang_base::{Identifier, element::Visibility};
pub use microcad_lang_types::ty::{MatrixType, QuantityType, TupleType, Ty, Unit};

use derive_more::{Deref, Display, From};
use microcad_lang_base::{CompactString, SrcRef, ToCompactString, VersionAnnotation};
use serde::{Deserialize, Serialize};

use crate::ir;

#[derive(Debug, Default, Display, Deref, Clone, Hash, PartialEq, Serialize, Deserialize)]
#[display("{}", ty)]
pub struct Type {
    #[deref]
    pub ty: microcad_lang_types::Type,
    pub src_ref: SrcRef,
}

impl From<microcad_lang_types::Type> for Type {
    fn from(ty: microcad_lang_types::Type) -> Self {
        Self {
            ty,
            src_ref: SrcRef::none(),
        }
    }
}

/// Symbol meta data
#[derive(Debug, Default, Hash, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub name: Option<Identifier>,

    /// Visibility
    pub vis: Visibility,

    /// Source code reference of the symbol definition
    pub src_ref: SrcRef,

    /// Source code reference of symbol's keyword
    pub keyword_src_ref: SrcRef,
}

#[derive(Debug, Hash, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alias {
    pub path: Path,
}

#[derive(Debug, Hash, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wildcard {
    pub path: Path,
}

#[derive(Debug, Hash, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineModule {}

#[derive(Debug, Hash, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileModule {}

/// A constant definition: `const FOO: Length = 32mm`.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Constant {
    /// Type of the constant
    pub ty: ir::Type,
    /// Expression type
    pub expr: ir::ConstantExpression,
}

impl Constant {
    pub fn value(&self) -> Option<&Value> {
        self.expr.value()
    }
}

/// IR item definition
#[derive(Debug, Clone, Hash, From, PartialEq, Serialize, Deserialize, IntoStaticStr)]
pub enum Def {
    /// Source file symbol.
    Source(Source),
    /// Inline Module symbol: `mod foo {}`
    InlineModule(InlineModule),
    /// File Module Symbol: `mod foo;`
    FileModule(FileModule),
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
pub struct Item {
    /// Item metadata
    pub meta: Meta,
    /// Item definition
    pub def: Def,
    /// Item documentation
    pub doc: DocBlock,
    /// Item version annotation
    pub ver: VersionAnnotation,
}

impl Item {
    pub fn name(&self) -> CompactString {
        self.meta
            .name
            .as_ref()
            .cloned()
            .unwrap_or_default()
            .to_compact_string()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    root: NodeId,
    arena: Arena,
}

impl From<(NodeId, Arena)> for Tree {
    fn from(value: (NodeId, Arena)) -> Self {
        Self {
            root: value.0,
            arena: value.1,
        }
    }
}

impl std::hash::Hash for Tree {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root().descendants().for_each(|node| node.hash(state));
    }
}

impl Tree {
    pub fn new(root: Item) -> Self {
        let mut arena = Arena::default();
        let root = arena.new_node(root);
        Self { root, arena }
    }
    pub fn root<'a>(&'a self) -> NodeRef<'a> {
        NodeRef::new(self.root, &self.arena)
    }

    pub fn root_mut<'a>(&'a mut self) -> NodeMut<'a> {
        NodeMut::new(self.root, &mut self.arena)
    }

    pub fn arena(&self) -> &Arena {
        &self.arena
    }

    pub fn root_id(&self) -> NodeId {
        self.root
    }
}

pub type Arena = microcad_lang_base::tree::Arena<Item>;
pub type Node = microcad_lang_base::tree::Node<Item>;
pub type NodeRef<'a> = microcad_lang_base::tree::NodeRef<'a, Item>;
pub type NodeMut<'a> = microcad_lang_base::tree::NodeMut<'a, Item>;
pub type NodeId = microcad_lang_base::tree::NodeId;
