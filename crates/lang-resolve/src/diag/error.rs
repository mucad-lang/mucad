// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A resolve error

use microcad_lang_base::{HashId, SrcRef, SrcReferrer, element::Case};

use microcad_lang_lower::LowerError;
use microcad_lang_parse::ParseError;
use microcad_lang_types::Type;
use microcad_std::StdLibError;
use miette::Diagnostic;
use thiserror::Error;

use crate::{ManifestError, locate::LocateError};

#[derive(Debug, Error, Diagnostic)]
pub enum ResolveErrorKind {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("{0}")]
    LowerError(#[from] LowerError),

    #[error("{0}")]
    ParseError(#[from] ParseError),

    #[error("{0}")]
    Locate(#[from] LocateError),

    #[error("{0}")]
    Manifest(#[from] ManifestError),

    #[error("{0}")]
    StdLib(#[from] StdLibError),

    #[error("Wrong case")]
    #[diagnostic(severity = "warning")]
    WrongCase {
        expected: Case,
        actual: Case,
        #[label]
        src_ref: SrcRef,
    },
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

    #[error("Error compiling file: {}", path.display())]
    CompileError { path: std::path::PathBuf },

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

    #[error("Library `{name}` not loaded")]
    LibraryNotLoaded { name: String },
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
            WrongCase { src_ref, .. } => *src_ref,
            TypeMismatch {
                specified_src_ref, ..
            } => *specified_src_ref,
            UnresolvedPath { src_ref, .. } | AmbiguousPath { src_ref, .. } => *src_ref,
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
