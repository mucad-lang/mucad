// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Data structure for the package manifest, saved as `mu.toml`.

use mucad_lang_base::{FileSystem, Name, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Manifest error.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// An IO error while processing the `manifest.toml` file.
    #[error("I/O error while reading manifest")]
    Io(#[from] std::io::Error),

    /// A TOML parse error.
    #[error("failed to parse manifest")]
    TomlDeserialize(#[from] toml::de::Error),

    /// A TOML write error.
    #[error("failed to write manifest")]
    TomlSerialize(#[from] toml::ser::Error),

    /// The `mu.toml` file does not exist in the path.
    #[error("`mu.toml` not found in {path}")]
    NotFound { path: std::path::PathBuf },
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Manifest {
    pub library: LibrarySection,
    pub dependencies: Option<std::collections::BTreeMap<String, Dependency>>,
}

/// `package` descriptor.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct LibrarySection {
    /// Mandatory library name.
    pub name: Name,
    /// A short description of the library.
    pub description: Option<String>,
    /// Mandatory library version.
    pub version: semver::Version,
    /// Authors of the library.
    pub authors: Option<Vec<String>>,
    /// Documentation URL
    pub documentation: Option<Url>,
    /// License
    pub license: Option<String>,
    /// Do not load the standard library by default.
    pub no_std: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Dependency {
    pub path: Option<std::path::PathBuf>,
}

impl Manifest {
    pub const MU_TOML: &str = "mu.toml";

    /// Load the `mu.toml` at `path` from the file system `fs`.
    pub fn load(
        fs: &dyn FileSystem,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, ManifestError> {
        let manifest_path = path.as_ref();
        if !fs.is_file(manifest_path) {
            return Err(ManifestError::NotFound {
                path: manifest_path.to_path_buf(),
            });
        }

        let buf = fs.read_to_string(manifest_path)?;
        Ok(toml::from_str(&buf)?)
    }

    /// Save a `mu.toml` inside a path.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), ManifestError> {
        use std::io::Write;
        let s = toml::to_string(&self)?;
        let mut file = std::fs::File::create(path.as_ref())?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}
