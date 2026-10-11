// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver prelude
//!
//! Preferably include with: `use microcad_driver::prelude as mu;`
//! To use the traits: `use mu::traits::*;`

pub mod core {
    pub use microcad_core::*;
}

pub use core::{Color, Scalar};

pub mod base {
    pub use microcad_lang_base::*;
}

pub use base::{
    Artifact, ArtifactKind, Diagnostic, Diagnostics, HashId, HashMap, HashSet, Hashed, Identifier,
    IssueResult, Refer, Source, SourceKind, SourceLocation, SrcRef, StageResult, TextEdit, Url,
};

pub mod builtin {
    pub use microcad_builtin::*;
}

pub mod parse {
    pub use microcad_lang_parse::*;
}

pub use parse::{Ast, ast, parse};

pub mod lower {
    pub use microcad_lang_lower::*;
}

pub use lower::{Ir, ir, lower};

pub mod resolve {
    pub use microcad_lang_resolve::*;
}

pub use resolve::{Library, ResolveContext, ResolveLibraryContext};

pub mod eval {
    pub use microcad_lang_eval::*;
}

pub use eval::EvalContext;

pub mod types {
    pub use microcad_lang_types::*;
}

pub use types::{ModelTree, Value};

pub use crate::Compiler;

pub use crate::config::DriverConfig;
pub use crate::document::{Document, SourceFile};
//pub use crate::session::Session;
pub use crate::watcher::Watcher;

pub use crate::{Cached, Report, Result, report};

pub use crate::commands;
pub use crate::document;

pub use resolve::locate;

pub use microcad_lang_format::format;

pub use crate::commands::{
    CompileParameters, Format, FormatParameters, PrintDiagnosticsParameters, Sync, compile::Parse,
    compile::ResolveParameters,
};

pub mod traits {
    pub use super::base::{Identifiable, SrcReferrer, ToHash};
    pub use super::core::{CalcBounds2D, CalcBounds3D};

    pub use super::commands::{
        Compile, Format, GetCode, PrintDiagnostics, SetCode, Sync, compile::Eval, compile::Lower,
        compile::Parse, compile::Resolve,
    };
}
