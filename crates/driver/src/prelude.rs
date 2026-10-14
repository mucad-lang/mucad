// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver prelude
//!
//! Preferably include with: `use microcad_driver::prelude as mu;`
//! To use the traits: `use mu::traits::*;`

// =========================================================================
// --- Core & Geometry ---
// =========================================================================

/// Symbols from `microcad_core`
pub mod core {
    pub use microcad_core::*;
}

pub use core::{Color, Scalar};

// =========================================================================
// --- Language Infrastructure & Pipeline ---
// =========================================================================

/// Symbols from `microcad_base`
pub mod base {
    pub use microcad_lang_base::*;
}

pub use base::{
    Artifact, ArtifactKind, Diagnostic, Diagnostics, HashId, HashMap, HashSet, Hashed, Identifier,
    IssueResult, MUCAD_EXTENSION, MUCAD_EXTENSIONS, MUCAD_VERSION, Refer, Shared, SharedMut,
    Source, SourceKind, SourceLocation, SrcRef, StageResult, TextEdit, Url,
};

/// Symbols from `microcad_lang_parse`
pub mod parse {
    pub use microcad_lang_parse::*;
}

pub use parse::{Ast, ast, parse};

/// Symbols from `microcad_lang_lower`
pub mod lower {
    pub use microcad_lang_lower::*;
}

pub use lower::{Ir, ir, lower};

/// Symbols from `microcad_lang_resolve`
pub mod resolve {
    pub use microcad_lang_resolve::*;
}

pub use resolve::{Library, ResolveContext, ResolveLibraryContext, locate};

/// Symbols from `microcad_lang_eval`
pub mod eval {
    pub use microcad_lang_eval::*;
}

pub use eval::EvalContext;

/// Symbols from `microcad_lang_types`
pub mod types {
    pub use microcad_lang_types::*;
}

pub use types::{ModelTree, Value};

// =========================================================================
// --- Built-in and standard library & Documentation Generation ---
// =========================================================================

/// Symbols from `microcad_builtin`
pub mod builtin {
    pub use microcad_builtin::*;
}

pub use builtin::BuiltinLibrary;

pub mod stdlib {
    pub use microcad_std::*;

    use super::{Library, ResolveContext, Shared};

    /// Load the standard library.
    ///
    /// TODO: Error handling.
    pub fn load(path: impl AsRef<std::path::Path>) -> Shared<Library> {
        let mut resolve_context = ResolveContext::new();
        let (stdlib_id, _) = resolve_context.load_external_library(path).unwrap();

        resolve_context
            .lib_cache()
            .with_read(|lib_cache| lib_cache.get(&stdlib_id))
            .unwrap()
    }
}

/// Symbols from `microcad_docgen`
pub mod docgen {
    pub use microcad_docgen::*;
}

// =========================================================================
// --- Driver Core, Configs & Commands ---
// =========================================================================

pub use crate::config::DriverConfig;
pub use crate::watcher::Watcher;
pub use crate::{Compiler, compiler::CompileResult};

pub use crate::{Report, Result, report};

pub mod format {
    pub use microcad_lang_format::*;
}

pub use crate::Formatter;
pub use format::{FormatConfig, Formatted, format};

/// Symbols from `microcad_render`
pub mod render {
    pub use microcad_render::*;
}

pub use render::RenderResolution;

/// Symbols from `microcad_export`
pub mod export {
    pub use microcad_export::*;
}

pub use crate::{Exporter, config::ExportConfig};

// =========================================================================
// --- Traits Namespace ---
// =========================================================================

/// Common traits across all µcad layers.
///
/// Import with: `use microcad_driver::prelude::traits::*;`
pub mod traits {
    pub use super::{
        base::{Identifiable, SrcReferrer, ToHash},
        core::{CalcBounds2D, CalcBounds3D},
        docgen::GenerateDoc,
    };
}
