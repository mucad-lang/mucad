// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module to handle IDs for any item used in µcad language.

use compact_str::ToCompactString;
use derive_more::Display;
use mucad_hash::{HashId, hash_id};
use serde::{Deserialize, Serialize};

use crate::{Name, Version};

/// A library id, constructed from name and version info of a library.
#[derive(
    Debug, Copy, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct LibraryId(HashId);

impl LibraryId {
    /// Reserved ID for the main crate/library currently being compiled.
    pub const ROOT: Self = LibraryId(HashId::new(0));

    pub fn is_root(self) -> bool {
        self == Self::ROOT
    }
}

impl From<LibraryInfo> for LibraryId {
    fn from(info: LibraryInfo) -> Self {
        info.id()
    }
}

/// A struct that holds metadata of library so the library can have a unique ID.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryInfo {
    /// Name of the library.
    pub name: Name,
    /// Version of the library.
    pub ver: Version, // TODO Check if a VersionAnnotation is useful here.
}

impl LibraryInfo {
    pub fn new(name: impl AsRef<str>, ver: Version) -> Self {
        Self {
            name: name.as_ref().to_compact_string(),
            ver,
        }
    }

    /// A unique ID for the library
    pub fn id(&self) -> LibraryId {
        LibraryId(hash_id!(&self.name, &self.ver))
    }

    pub fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("{}-{}", self.name, self.ver))
    }
}

impl std::fmt::Display for LibraryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.ver)
    }
}

impl std::fmt::Debug for LibraryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}[{}]", self.name, self.ver, self.id())
    }
}
