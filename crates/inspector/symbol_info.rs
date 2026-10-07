// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::rc::Rc;

use microcad_driver::prelude as mu;
use mu::traits::*;

/// Retrieve symbol information.
pub trait Info {
    /// Get symbol information.
    fn info(&self) -> SymbolInfo;
}

impl Info for mu::Creator {
    fn info(&self) -> SymbolInfo {
        self.symbol.info()
    }
}

impl Info for mu::Symbol {
    fn info(&self) -> SymbolInfo {
        self.with_def(|def| def.info())
    }
}

impl Info for mu::SymbolDef {
    fn info(&self) -> SymbolInfo {
        match self {
            Self::Root => unreachable!(),
            Self::SourceFile(sf) => sf.into(),
            Self::Module(md) => md.into(),
            Self::Workbench(wd) => wd.into(),
            Self::Function(fd) => fd.into(),
            Self::Builtin(bi) => bi.into(),
            Self::Assignment(a) => a.into(),

            Self::Value(id, ..) => SymbolInfo::new_local(id),

            _ => unimplemented!(),
        }
    }
}

/// Single parameter information in human readable form.
pub struct ParameterInfo {
    /// Parameter ID.
    pub id: String,
    /// Type if given.
    pub ty: Option<String>,
    /// Default value if given.
    pub def: Option<String>,
}

impl From<&mu::ir::Parameter> for ParameterInfo {
    fn from(param: &mu::ir::Parameter) -> Self {
        Self {
            id: param.id().to_string(),
            ty: param.ty.clone().map(|ty| ty.to_string()),
            def: param.default_value.clone().map(|def| def.to_string()),
        }
    }
}

impl From<(&mu::Identifier, &mu::ParameterValue)> for ParameterInfo {
    fn from(param: (&mu::Identifier, &mu::ParameterValue)) -> Self {
        Self {
            id: param.0.to_string(),
            ty: param.1.specified_type.clone().map(|ty| ty.to_string()),
            def: param.1.default_value.clone().map(|def| def.to_string()),
        }
    }
}

/// Call signature information in human readable form.
pub struct SignatureInfo {
    /// Parameters of the call.
    pub params: Vec<ParameterInfo>,
    /// Documentation.
    pub doc: Option<mu::ir::DocBlock>,
}

/// Symbol information in human readable form.
#[derive(Default)]
pub struct SymbolInfo {
    /// ID of the symbol.
    pub id: String,
    /// Human readable symbol kind.
    pub kind: String,
    /// Optional documentation block.
    pub doc: Option<mu::ir::DocBlock>,
    /// Parameters and alternative parameters if any.
    pub signatures: Vec<SignatureInfo>,
    /// Source code reference.
    pub src_ref: mu::SrcRef,
}

impl SymbolInfo {
    pub(super) fn new_local(id: &mu::Identifier) -> Self {
        SymbolInfo {
            id: id.to_string(),
            kind: "local value".into(),
            ..Default::default()
        }
    }
}

impl From<&Rc<mu::ir::LocalAssignment>> for SymbolInfo {
    fn from(def: &Rc<mu::ir::LocalAssignment>) -> Self {
        SymbolInfo {
            id: def.id().to_string(),
            kind: "Assignment".into(),
            doc: Some(def.doc()),
            signatures: vec![],
            src_ref: def.src_ref(),
        }
    }
}

impl From<&Rc<mu::ir::Source>> for SymbolInfo {
    fn from(def: &Rc<mu::ir::Source>) -> Self {
        SymbolInfo {
            id: def.id().to_string(),
            kind: "SourceFile".into(),
            doc: Some(def.doc()),
            signatures: vec![],
            src_ref: def.src_ref(),
        }
    }
}

impl From<&Rc<mu::ir::ModuleDefinition>> for SymbolInfo {
    fn from(def: &Rc<mu::ir::ModuleDefinition>) -> Self {
        SymbolInfo {
            id: def.id().to_string(),
            kind: "ModuleDefinition".into(),
            doc: Some(def.doc()),
            signatures: vec![],
            src_ref: def.src_ref(),
        }
    }
}

impl From<&Rc<mu::ir::WorkbenchDefinition>> for SymbolInfo {
    fn from(def: &Rc<mu::ir::WorkbenchDefinition>) -> Self {
        use mu::lower::Initialized;

        SymbolInfo {
            id: def.id().to_string(),
            kind: def.kind.to_string(),
            doc: Some(def.doc()),
            signatures: def
                .inits()
                .map(|init| SignatureInfo {
                    params: init.parameters.iter().map(|p| p.into()).collect(),
                    doc: Some(init.doc()),
                })
                .collect(),
            src_ref: def.src_ref(),
        }
    }
}

impl From<&Rc<mu::ir::FunctionDefinition>> for SymbolInfo {
    fn from(def: &Rc<mu::ir::FunctionDefinition>) -> Self {
        SymbolInfo {
            id: def.id().to_string(),
            kind: "Function".into(),
            doc: Some(def.doc()),
            signatures: vec![SignatureInfo {
                params: def.signature.parameters.iter().map(|p| p.into()).collect(),
                doc: Some(def.doc()),
            }],
            src_ref: def.src_ref(),
        }
    }
}

impl From<&mu::Builtin> for SymbolInfo {
    fn from(def: &mu::Builtin) -> Self {
        SymbolInfo {
            id: def.id().to_string(),
            kind: "Builtin".into(),
            doc: Some(def.doc()),
            signatures: vec![],
            src_ref: mu::SrcRef::none(),
        }
    }
}
