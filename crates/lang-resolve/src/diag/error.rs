// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A resolve error

use microcad_lang_base::{HashId, SrcRef, SrcReferrer};

use microcad_lang_lower::LowerError;
use microcad_lang_parse::ParseError;
use microcad_lang_types::Type;
use miette::Diagnostic;
use thiserror::Error;

use crate::{ManifestError, locate::LocateError};

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveErrorKind {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LowerError(#[from] LowerError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(#[from] ParseError),

    #[error("{0}")]
    Locate(#[from] LocateError),

    #[error("{0}")]
    Manifest(#[from] ManifestError),

    #[error("Type mismatch: {specified} != {actual}")]
    TypeMismatch {
        specified: Type,
        #[label("Specified type")]
        specified_src_ref: SrcRef,
        actual: Type,
        #[label("Actual type")]
        actual_src_ref: SrcRef,
    },
    #[error("No source with hash: {0}")]
    NoSourceWithHash(HashId),

    #[error("Library entry point found at {}", lib_mu.display())]
    NoLibraryEntryPoint { lib_mu: std::path::PathBuf },

    #[error("Source '{0}' has no path.")]
    SourceHasNoPath(String),

    #[error("Unresolved symbol: {path}")]
    UnresolvedPath {
        path: String,
        #[label("This symbol could not be found")]
        src_ref: SrcRef,
    },

    #[error("Ambiguous symbol: {path}")]
    #[diagnostic(help("Candidates: {}", candidates.join(", ")))]
    AmbiguousPath {
        path: String,
        candidates: Vec<String>,
        #[label("This symbol matches multiple definitions")]
        src_ref: SrcRef,
    },

    /// Assignments in sources and workbenches are properties of the model with unique names.
    #[error("Property `{name}` is already defined in this scope")]
    #[diagnostic(help(
        "Assignments in sources and workbenches are properties of the model, use another name."
    ))]
    PropertyAlreadyDefined {
        name: String,
        #[label("Property defined again")]
        src_ref: SrcRef,
    },

    /// The symbols of a module or workbench have unique names.
    #[error("Symbol `{name}` is already defined in this scope")]
    #[diagnostic(help("Rename one of the symbols."))]
    SymbolAlreadyDefined {
        name: String,
        #[label("Symbol defined again")]
        src_ref: SrcRef,
        #[label("First definition")]
        first_src_ref: SrcRef,
    },

    /// An alias stands for itself through other aliases, e.g. `use d as c; use c as d;`.
    #[error("Cyclic alias `{name}`")]
    #[diagnostic(help("The alias refers to itself through other aliases."))]
    CyclicAlias {
        name: String,
        #[label("This alias does not refer to a symbol")]
        src_ref: SrcRef,
    },

    #[error("Library `{name}` not loaded")]
    LibraryNotLoaded { name: String },

    /// Libraries must not depend on each other.
    #[error("Cyclic library dependency: {chain}")]
    CyclicDependency { chain: String },

    /// Only libraries with a manifest can be used as dependency, because the manifest defines the library ID.
    #[error("Library has no manifest (`mu.toml`) and cannot be used as dependency")]
    LibraryWithoutManifest,
}

#[derive(Debug)]
pub struct ResolveError(pub Box<ResolveErrorKind>);

impl ResolveError {
    pub fn new(err: impl Into<ResolveErrorKind>) -> Self {
        Self(Box::new(err.into()))
    }

    pub fn kind(&self) -> &ResolveErrorKind {
        &self.0
    }
}

impl From<ResolveErrorKind> for ResolveError {
    fn from(kind: ResolveErrorKind) -> Self {
        Self(Box::new(kind))
    }
}

impl From<ParseError> for ResolveError {
    fn from(err: ParseError) -> Self {
        ResolveErrorKind::from(err).into()
    }
}

impl From<LowerError> for ResolveError {
    fn from(err: LowerError) -> Self {
        ResolveErrorKind::from(err).into()
    }
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

// 2. Delegate miette::Diagnostic to the inner kind
impl Diagnostic for ResolveError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.0.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.0.source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.0.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.0.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.0.diagnostic_source()
    }
}

impl SrcReferrer for ResolveError {
    fn src_ref(&self) -> SrcRef {
        use ResolveErrorKind::*;
        match &*self.0 {
            TypeMismatch {
                specified_src_ref, ..
            } => *specified_src_ref,
            UnresolvedPath { src_ref, .. }
            | AmbiguousPath { src_ref, .. }
            | PropertyAlreadyDefined { src_ref, .. }
            | CyclicAlias { src_ref, .. }
            | SymbolAlreadyDefined { src_ref, .. } => *src_ref,
            ParseError(err) => err.src_ref,
            LowerError(err) => err.src_ref(),
            _ => SrcRef::none(),
        }
    }
}

impl From<std::io::Error> for ResolveError {
    fn from(value: std::io::Error) -> Self {
        Self(Box::new(value.into()))
    }
}

impl From<LocateError> for ResolveError {
    fn from(value: LocateError) -> Self {
        Self(Box::new(value.into()))
    }
}
