// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A compiler artifact to be persisted, e.g. an IR written and read from file.

use derive_more::{Display, FromStr};
use mucad_hash::HashId;
use miette::Diagnostic as MietteDiagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Diagnostics, Issue, IssueList, IssueResult, LanguageVersion, ToDiagnostics};

#[derive(Debug, Error, MietteDiagnostic)]
pub enum ArtifactError {
    /// Postcard error
    #[error("Binary serialization error")]
    BinarySerialization(#[from] postcard::Error),

    /// Error during serializing to RON.
    #[error("RON serialization error")]
    RonSerialization(#[from] ron::Error),

    /// Error during deserializing from RON.
    #[error("RON deserialization error")]
    RonDeserialization(#[from] ron::error::SpannedError),

    /// Wrong artifact kind
    #[error(": {expected} != {got}")]
    WrongArtifactKind {
        expected: ArtifactKind,
        got: ArtifactKind,
    },

    /// Version mismatch
    #[error("Version mismatch: {expected} != {got}")]
    VersionMismatch {
        expected: LanguageVersion,
        got: LanguageVersion,
    },
}

#[derive(Debug, Display, PartialEq, Clone, Copy, Serialize, Deserialize, FromStr)]
#[from_str(rename_all = "lowercase")]
pub enum ArtifactKind {
    /// Abstract syntax tree
    Ast,
    /// Intermediate representation
    Ir,
    /// Resolved Library
    Library,
    /// Model tree
    ModelTree,
}

impl ArtifactKind {
    pub fn ext(&self) -> &'static str {
        match self {
            ArtifactKind::Ast => "ast",
            ArtifactKind::Ir => "ir",
            ArtifactKind::Library => "lib",
            ArtifactKind::ModelTree => "model",
        }
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactHeader {
    pub magic: [u8; 4],           // "&mu;"
    pub version: LanguageVersion, // Increment this whenever the format changes
    pub kind: ArtifactKind,       // The artifact type (AST, IR, etc.)
}

impl From<ArtifactKind> for ArtifactHeader {
    fn from(kind: ArtifactKind) -> Self {
        Self {
            magic: *b"&mu;",
            version: LanguageVersion::current(),
            kind,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EnvelopeRef<'artifact, T> {
    pub header: ArtifactHeader,
    pub payload: &'artifact T,
}

#[derive(Debug, Deserialize)]
pub struct Envelope<T> {
    pub header: ArtifactHeader,
    pub payload: T,
}

impl<T> Envelope<T> {
    pub fn unwrap(self, kind: ArtifactKind) -> Result<T, ArtifactError> {
        if kind != self.header.kind {
            return Err(ArtifactError::WrongArtifactKind {
                expected: kind,
                got: self.header.kind,
            });
        }

        let version = self.header.version;
        if !version.is_compatible() {
            return Err(ArtifactError::VersionMismatch {
                expected: LanguageVersion::current(),
                got: version,
            });
        }

        Ok(self.payload)
    }
}

/// Trait to define a compilation artifact.
pub trait Artifact: Sized + Serialize {
    fn kind() -> ArtifactKind;

    /// Return the source hash of this artifact
    fn source_hash(&self) -> HashId {
        Default::default()
    }

    /// Envelope this artifact with a header.
    fn envelope(&'_ self) -> EnvelopeRef<'_, Self> {
        EnvelopeRef {
            header: Self::kind().into(),
            payload: self,
        }
    }

    /// Load the artifact from binary
    fn from_binary<'de>(bytes: &'de [u8]) -> Result<Self, ArtifactError>
    where
        Self: Deserialize<'de>,
    {
        let envelope: Envelope<Self> = postcard::from_bytes(bytes)?;
        envelope.unwrap(Self::kind())
    }

    /// Convert the artifact into binary
    fn to_binary(&self) -> Result<Vec<u8>, ArtifactError> {
        Ok(postcard::to_allocvec(&self.envelope())?)
    }

    /// Convert the artifact into a Rusty Object Notation
    fn to_ron(&self) -> Result<String, ArtifactError> {
        // Configure indentation, spacing, and multi-line breaks
        let config = ron::ser::PrettyConfig::default()
            .indentor("    ".to_string()) // Beautiful 4-space indent
            .new_line("\n".to_string());

        Ok(ron::ser::to_string_pretty(&self.envelope(), config)?)
    }

    /// Read the artifact from a Rusty Object Notation
    fn from_ron<'de>(s: impl Into<&'de str>) -> Result<Self, ArtifactError>
    where
        Self: Deserialize<'de>,
    {
        let envelope: Envelope<Self> = ron::de::from_str(s.into())?;
        envelope.unwrap(Self::kind())
    }

    /// Write the artifact into `path` with the extension of its kind, e.g. `circle.mu.ast`.
    ///
    /// Returns the path of the written file.
    #[cfg(feature = "io")]
    fn emit(&self, path: impl AsRef<std::path::Path>) -> miette::Result<std::path::PathBuf> {
        use miette::IntoDiagnostic;
        let mut path = path.as_ref().to_path_buf();
        path.add_extension(Self::kind().ext());
        std::fs::write(&path, self.to_ron()?).into_diagnostic()?;
        Ok(path)
    }
}

/// The result of a compilation stage.
#[derive(Debug)]
pub struct StageResult<T: Artifact, I: Issue>(Option<IssueResult<T, I>>);

impl<T: Artifact, I: Issue> StageResult<T, I> {
    pub fn new(result: IssueResult<T, I>) -> Self {
        Self(Some(result))
    }

    pub fn reset(&mut self) {
        self.0 = None;
    }

    /// Return a reference to artifact.
    pub fn artifact(&self) -> Option<&T> {
        self.0.as_ref()?.as_ref().ok().map(|(val, _)| val)
    }

    /// Fetch this artifact
    pub fn into_artifact(self) -> Option<T> {
        self.0?.ok().map(|(val, _)| val)
    }

    pub fn issues(&self) -> Option<&IssueList<I>> {
        match &self.0 {
            Some(Ok((_, issues))) | Some(Err(issues)) => Some(issues),
            None => None,
        }
    }

    /// Returns true if this compilation stage has been successful.
    pub fn is_ok(&self) -> bool {
        matches!(self.0, Some(Ok(_)))
    }
}

impl<T: Artifact, I: Issue + Clone> ToDiagnostics for StageResult<T, I> {
    fn to_diagnostics(&self) -> Diagnostics {
        match self.issues() {
            Some(issues) => issues.to_diagnostics(),
            None => Diagnostics::new(),
        }
    }
}

impl<T: Artifact, I: Issue> Default for StageResult<T, I> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Artifact, I: Issue> From<IssueResult<T, I>> for StageResult<T, I> {
    fn from(result: IssueResult<T, I>) -> Self {
        Self::new(result)
    }
}
