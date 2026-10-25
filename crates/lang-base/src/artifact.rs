// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A compiler artifact to be persisted, e.g. an IR written and read from file.

use derive_more::{Display, FromStr};
use miette::Diagnostic as MietteDiagnostic;
use mucad_hash::HashId;
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

/// The kind of an artifact, its byte in the binary [`ArtifactHeader`] is its discriminant.
#[repr(u8)]
#[derive(Debug, Display, PartialEq, Clone, Copy, Serialize, Deserialize, FromStr)]
#[from_str(rename_all = "lowercase")]
pub enum ArtifactKind {
    /// Abstract syntax tree
    Ast = 0,
    /// Intermediate representation
    Ir = 1,
    /// Resolved Library
    Library = 2,
    /// Model tree
    ModelTree = 3,
    /// Lexed tokens, e.g. of a shared link
    Tokens = 4,
}

impl ArtifactKind {
    pub fn ext(&self) -> &'static str {
        match self {
            ArtifactKind::Ast => "muast",
            ArtifactKind::Ir => "muir",
            ArtifactKind::Library => "mulib",
            ArtifactKind::ModelTree => "mumodel",
            ArtifactKind::Tokens => "mutokens",
        }
    }
}

impl TryFrom<u8> for ArtifactKind {
    type Error = u8;

    /// The kind with the discriminant `byte`, fails with the byte.
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        [
            Self::Ast,
            Self::Ir,
            Self::Library,
            Self::ModelTree,
            Self::Tokens,
        ]
        .into_iter()
        .find(|kind| *kind as u8 == byte)
        .ok_or(byte)
    }
}

/// The header of an artifact.
///
/// Binary, the header has 4 bytes: the magic `µ` as its code point U+00B5, the kind and the
/// major version in 4 bits each, the minor and the patch version, e.g. `b5 40 06 00` for tokens
/// of version 0.6.0. Human-readable, e.g. in RON, it is a struct with the magic as text.
#[derive(Debug, PartialEq)]
pub struct ArtifactHeader {
    /// [`ArtifactHeader::MAGIC`]
    pub magic: u8,
    /// Increment this whenever the format changes
    pub version: LanguageVersion,
    /// The artifact type (AST, IR, etc.)
    pub kind: ArtifactKind,
}

impl ArtifactHeader {
    /// The magic at the start of an artifact, `µ` as its code point U+00B5.
    pub const MAGIC: u8 = 0xb5;
    /// The size of the binary header in bytes.
    pub const SIZE: usize = 4;
}

impl From<ArtifactKind> for ArtifactHeader {
    fn from(kind: ArtifactKind) -> Self {
        Self {
            magic: Self::MAGIC,
            version: LanguageVersion::current(),
            kind,
        }
    }
}

/// The human-readable form of an [`ArtifactHeader`].
#[derive(Serialize, Deserialize)]
#[serde(rename = "ArtifactHeader")]
struct ReadableHeader {
    /// The magic as text, `"µ"`.
    magic: String,
    version: LanguageVersion,
    kind: ArtifactKind,
}

impl Serialize for ArtifactHeader {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{Error, SerializeTuple};

        if serializer.is_human_readable() {
            return ReadableHeader {
                magic: char::from(self.magic).to_string(),
                version: self.version.clone(),
                kind: self.kind,
            }
            .serialize(serializer);
        }

        let version = semver::Version::from(self.version.clone());
        let part = |part: u64, max: u64| match part <= max {
            true => Ok(part as u8),
            false => Err(S::Error::custom(format!(
                "version {version} has a part > {max}"
            ))),
        };
        let bytes = [
            self.magic,
            (self.kind as u8) << 4 | part(version.major, 15)?,
            part(version.minor, 255)?,
            part(version.patch, 255)?,
        ];
        // A tuple has no length, unlike bytes.
        let mut tuple = serializer.serialize_tuple(Self::SIZE)?;
        for byte in bytes {
            tuple.serialize_element(&byte)?;
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for ArtifactHeader {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let invalid_magic = || D::Error::custom("no µcad artifact, the magic `µ` is missing");
        if deserializer.is_human_readable() {
            let header = ReadableHeader::deserialize(deserializer)?;
            if header.magic != char::from(Self::MAGIC).to_string() {
                return Err(invalid_magic());
            }
            return Ok(Self {
                magic: Self::MAGIC,
                version: header.version,
                kind: header.kind,
            });
        }

        let [magic, kind_major, minor, patch] = <[u8; Self::SIZE]>::deserialize(deserializer)?;
        if magic != Self::MAGIC {
            return Err(invalid_magic());
        }
        let kind = ArtifactKind::try_from(kind_major >> 4)
            .map_err(|byte| D::Error::custom(format!("unknown artifact kind {byte}")))?;
        let major = kind_major & 0x0f;
        Ok(Self {
            magic: Self::MAGIC,
            version: semver::Version::new(major.into(), minor.into(), patch.into()).into(),
            kind,
        })
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

    /// Write the artifact into `path` with the extension of its kind, e.g. `circle.muast`.
    ///
    /// Returns the path of the written file.
    #[cfg(feature = "io")]
    fn emit(&self, path: impl AsRef<std::path::Path>) -> miette::Result<std::path::PathBuf> {
        use miette::IntoDiagnostic;
        let mut path = path.as_ref().to_path_buf();
        path.set_extension(Self::kind().ext());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MUCAD_VERSION;

    /// The binary header has 4 bytes: `µ`, the kind with the major version, the minor and
    /// the patch version.
    #[test]
    fn binary_header() {
        let header = ArtifactHeader {
            magic: ArtifactHeader::MAGIC,
            version: "1.6.2".parse().expect("A version"),
            kind: ArtifactKind::Tokens,
        };
        let bytes = postcard::to_allocvec(&header).expect("A binary header");
        assert_eq!(bytes, [0xb5, 0x41, 6, 2]);
        assert_eq!(char::from(bytes[0]), 'µ');
        assert_eq!(postcard::from_bytes::<ArtifactHeader>(&bytes), Ok(header));

        assert!(postcard::from_bytes::<ArtifactHeader>(&[b'&', 0x41, 6, 2]).is_err());
        assert!(postcard::from_bytes::<ArtifactHeader>(&[0xb5, 0xf0, 6, 2]).is_err());
        assert!(postcard::from_bytes::<ArtifactHeader>(&[0xb5, 0x41]).is_err());
        // A major version > 15 has no binary header.
        let header = ArtifactHeader {
            magic: ArtifactHeader::MAGIC,
            version: "16.0.0".parse().expect("A version"),
            kind: ArtifactKind::Tokens,
        };
        assert!(postcard::to_allocvec(&header).is_err());
    }

    /// The human-readable header has the magic as text.
    #[test]
    fn readable_header() {
        let header = ArtifactHeader::from(ArtifactKind::Ast);
        let ron = ron::to_string(&header).expect("A RON header");
        assert_eq!(
            ron,
            format!("(magic:\"µ\",version:(\"{MUCAD_VERSION}\"),kind:Ast)")
        );
        assert_eq!(ron::from_str::<ArtifactHeader>(&ron), Ok(header));
        assert!(ron::from_str::<ArtifactHeader>(&ron.replace('µ', "u")).is_err());
    }

    /// The kind of a byte is the kind with the byte as discriminant.
    #[test]
    fn kind_bytes() {
        for kind in [
            ArtifactKind::Ast,
            ArtifactKind::Ir,
            ArtifactKind::Library,
            ArtifactKind::ModelTree,
            ArtifactKind::Tokens,
        ] {
            assert_eq!(ArtifactKind::try_from(kind as u8), Ok(kind));
        }
        assert_eq!(ArtifactKind::try_from(5), Err(5));
    }
}
