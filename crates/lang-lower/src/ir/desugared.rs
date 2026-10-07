// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Item desugared from AST. These items will be scaffolded into the IR.
//!
//! All these items are crate-internal, because they further processed during the scaffolding step.
//! E.g., the attributes and meta will be further processed into item-specific structs.

use microcad_macros::Scaffold;
use serde::{Deserialize, Serialize};

use crate::ir;

/// `use std::geo2d::Circle as C` => (path = "std::geo2d::Circle", id = "C")
/// `use std::geo2d::Circle` => (path = "std::geo2d::Circle", id = "Circle")
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Alias {
    pub meta: ir::Meta,
    pub attr: ir::Attributes,
    pub path: ir::Path,
}

/// `use std::geo2d::*`
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Wildcard {
    pub meta: ir::Meta,
    pub attr: ir::Attributes,
    pub path: ir::Path,
}

/// Aliases lowered from `use` statements.
#[derive(Debug, Clone, Default, Hash, PartialEq, Serialize, Deserialize, Scaffold)]
pub(crate) struct Aliases {
    pub explicit_aliases: Box<[Alias]>,
    pub wildcards: Box<[Wildcard]>,
}

/// A constant definition: `const FOO: Length = 32mm`.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Constant {
    pub meta: ir::Meta,
    pub attr: ir::Attributes,
    pub ty: ir::Type,
    pub expr: ir::ConstantExpression,
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Serialize, Deserialize, Scaffold)]
pub(crate) struct FunctionItems {
    /// use ...
    pub aliases: Aliases,
    /// const FOO =
    pub constants: Box<[Constant]>,
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Function {
    pub meta: ir::Meta,
    pub attr: ir::Attributes,
    pub items: FunctionItems,

    pub signature: ir::FunctionSignature,

    /// Function statements
    pub statements: Box<[ir::FunctionStatement]>,
}

/// Inline module definition.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct InlineModule {
    pub meta: ir::Meta,
    pub attr: ir::Attributes,
    pub items: InlineModuleItems,
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct FileModule {
    pub meta: ir::Meta,
    pub attr: ir::Attributes,
}

/// Items inside an inline module that will be resolved into Symbols.
#[derive(Debug, Clone, Default, Hash, PartialEq, Serialize, Deserialize, Scaffold)]
pub struct InlineModuleItems {
    pub modules: Box<[InlineModule]>,
    pub aliases: Aliases,
    pub constants: Box<[Constant]>,
    pub functions: Box<[Function]>,
    pub workbenches: Box<[Workbench]>,
}

/// Workbench items that will be resolved into Symbols
#[derive(Debug, Clone, Default, Hash, PartialEq, Serialize, Deserialize, Scaffold)]
pub(crate) struct WorkbenchItems {
    /// `use`
    pub aliases: Aliases,
    /// `const`
    pub constants: Box<[Constant]>,
    /// `fn`
    pub functions: Box<[Function]>,
}

/// Workbench definition, e.g `sketch`, `part` or `op`.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Workbench {
    pub meta: ir::Meta,
    /// Attributes
    pub attr: ir::Attributes,
    /// Items that will be resolved into Symbols
    pub items: WorkbenchItems,
    /// Workbench kind.
    pub kind: ir::WorkbenchKind,
    /// Workbench's building plan.
    pub parameters: ir::ParameterList,
    /// `init`
    pub inits: Box<[ir::Init]>,
    /// The actual statements to build the Model
    pub statements: Box<[ir::WorkbenchStatement]>,
}

/// Items of a source file that will become symbols.
#[derive(Debug, Clone, Default, Hash, PartialEq, Serialize, Deserialize, Scaffold)]
pub(crate) struct SourceItems {
    /// List of file modules: `mod foo;`.
    pub file_modules: Box<[FileModule]>,
    /// Inline modules: `mod bar {...}`.
    pub inline_modules: Box<[InlineModule]>,
    /// Use statements: `use ...`.
    pub aliases: Aliases,
    /// Constants: `const FOO = 42;`.
    pub constants: Box<[Constant]>,
    /// Functions: `fn foo(...) {...}`.
    pub functions: Box<[Function]>,
    /// Workbenches: `part Bar(...) {...}`.
    pub workbenches: Box<[Workbench]>,
}

/// A desugared source file.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Source {
    /// Meta data.
    pub meta: ir::Meta,
    /// Attributes
    pub attr: ir::Attributes,
    /// Items that will become Symbols
    pub items: SourceItems,
    /// Workbench statements
    pub statements: Box<[ir::SourceStatement]>,
}
