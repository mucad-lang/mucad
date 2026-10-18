// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A compiler artifact to be persisted, e.g. an IR written and read from file.

use mucad_lang_base::{Identifier, SrcRef};
use serde::{Deserialize, Serialize};

use crate::ir;

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExportAttribute {
    pub file: ir::ConstantExpression,
}

/// A statement in a source file.
///
/// Source file statements are similar to workbench statements with the following features
/// * No properties `prop` allowed
/// * Export attributes
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SourceStatement {
    pub exports: Box<[ExportAttribute]>,
    pub attr: ir::ModelAttributes,
    pub src_ref: SrcRef,
    pub name: Option<ir::Identifier>,
    pub ty: ir::Type,
    pub expression: ir::WorkbenchExpression,
}

/// Builder methods for testing
impl SourceStatement {
    /// Create an assignment statement: `id = expr;`
    pub fn assignment(id: impl AsRef<str>, expr: impl Into<ir::WorkbenchExpression>) -> Self {
        SourceStatement {
            exports: Box::default(),
            attr: Default::default(),
            src_ref: Default::default(),
            name: Some(Identifier::from(id.as_ref())),
            ty: Default::default(),
            expression: expr.into(),
        }
    }

    /// Create an expression statement: `expr;`
    pub fn expr(expr: impl Into<ir::WorkbenchExpression>) -> Self {
        SourceStatement {
            exports: Box::default(),
            attr: Default::default(),
            src_ref: Default::default(),
            name: None,
            ty: Default::default(),
            expression: expr.into(),
        }
    }
}

/// A desugared source file.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Source {
    /// Workbench statements
    pub statements: Box<[ir::SourceStatement]>,
}
