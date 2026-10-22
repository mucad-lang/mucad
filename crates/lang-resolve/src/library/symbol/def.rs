// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Symbol Definitions in a library.
//!
//! These definition are mostly re-exported from IR or type-aliases, but they might become distinct types in the future.

use derive_more::From;
use mucad_lang_base::element::Case;
use mucad_lang_lower::ir;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

/// Function definitions from IR.
pub mod function {
    use mucad_lang_lower::ir;

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
    use mucad_lang_lower::ir;

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

/// Constant definitions.
pub mod constant {
    use mucad_lang_lower::ir;

    pub use ir::{Constant, ConstantExpression, ConstantValue};

    pub type Argument = ir::Argument<ir::ConstantExpression>;
    pub type ArgumentList = ir::ArgumentList<ir::ConstantExpression>;
}

pub use ir::{
    Alias, Argument, ArgumentList, Call, Constant, ConstantExpression, ConstantValue, DocBlock,
    ExportAttribute, ExprSpec, InlineModule, Meta, Parameter, ParameterList, Path, Source,
    SourceStatement, Visibility, Wildcard,
};

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

impl SymbolDef {
    /// The case of the name of a symbol, e.g. `UPPER_SNAKE_CASE` for constants.
    pub fn expected_case(&self) -> Option<Case> {
        use SymbolDef::*;
        match &self {
            InlineModule(_) | SourceFile(_) | Function(_) => Some(Case::LowerSnake),
            Workbench(workbench) => match workbench.signature.kind {
                WorkbenchKind::Sketch | WorkbenchKind::Part => Some(Case::Pascal),
                WorkbenchKind::Op => Some(Case::LowerSnake),
            },
            Constant(_) => Some(Case::UpperSnake),
            Alias(_) | Wildcard(_) | Root(_) => None,
        }
    }

    /// The parameter lists of a symbol: of a workbench (building plan and inits) or a function.
    pub fn parameter_lists(&self) -> Vec<&ParameterList> {
        match &self {
            SymbolDef::Workbench(workbench) => std::iter::once(&workbench.signature.parameters)
                .chain(
                    workbench
                        .signature
                        .inits
                        .iter()
                        .map(|init| &init.parameters),
                )
                .collect(),
            SymbolDef::Function(function) => vec![&function.signature.parameters],
            _ => Vec::new(),
        }
    }

    /// The parameter lists of a symbol to change them, see [`parameter_lists`].
    pub fn parameter_lists_mut(&mut self) -> Vec<&mut ir::ParameterList> {
        match self {
            SymbolDef::Workbench(workbench) => std::iter::once(&mut workbench.signature.parameters)
                .chain(
                    workbench
                        .signature
                        .inits
                        .iter_mut()
                        .map(|init| &mut init.parameters),
                )
                .collect(),
            SymbolDef::Function(function) => vec![&mut function.signature.parameters],
            _ => Vec::new(),
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
