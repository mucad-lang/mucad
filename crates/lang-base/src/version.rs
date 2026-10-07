// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// The version string from cargo.
pub const MICROCAD_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use semver::Version;

use crate::DisplayOneLine;

/// Storage for µcad language version
#[derive(Debug, Display, PartialEq, Serialize, Deserialize)]
#[display("{_0}")]
pub struct LanguageVersion(Version);

impl LanguageVersion {
    /// The current µcad language version
    pub fn current() -> Self {
        Self(Version::parse(MICROCAD_VERSION).expect("A valid cargo package version"))
    }

    /// Checks if a version string is compatible with the current µcad compiler
    pub fn is_compatible(&self) -> bool {
        use semver::VersionReq;

        // Use caret requirements: ^0.1.0 allows 0.1.x but not 0.2.0
        let req = VersionReq::parse(&format!("^{}", MICROCAD_VERSION)).unwrap();

        req.matches(&self.0)
    }
}

impl From<LanguageVersion> for Version {
    fn from(value: LanguageVersion) -> Self {
        value.0
    }
}

impl std::str::FromStr for LanguageVersion {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(semver::Version::parse(s)?))
    }
}

/// Represents the stability level of a symbol in µcad.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stability {
    Experimental,
    Stable,
    Deprecated {
        since: Option<Version>,
        note: Option<String>,
        removal_in: Option<Version>,
    },
}

/// Version annotation for a symbol in µcad.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionAnnotation {
    /// The version when the symbol was introduced to the DSL.
    pub introduced: Option<Version>,

    /// Current stability state of the symbol.
    pub stability: Stability,
}

impl VersionAnnotation {
    /// Helper for stable/core symbols without a specific version lock
    pub const STABLE: Self = Self {
        introduced: None,
        stability: Stability::Stable,
    };

    /// Checks if the symbol is available at a given target DSL version.
    pub fn is_available_at(&self, target_version: &Version) -> bool {
        match &self.introduced {
            Some(introduced) => target_version >= introduced,
            None => true,
        }
    }

    /// Checks if the symbol is deprecated at a given target DSL version.
    pub fn is_deprecated_at(&self, target_version: &Version) -> bool {
        match &self.stability {
            Stability::Deprecated {
                since: Some(since), ..
            } => target_version >= since,
            Stability::Deprecated { since: None, .. } => true,
            _ => false,
        }
    }
}

impl Default for VersionAnnotation {
    fn default() -> Self {
        Self::STABLE
    }
}

impl std::fmt::Display for VersionAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 1. Format the stability status
        match &self.stability {
            Stability::Stable => write!(f, "stable")?,
            Stability::Experimental => write!(f, "experimental")?,
            Stability::Deprecated {
                since,
                note,
                removal_in,
            } => {
                write!(f, "deprecated")?;

                let mut parts = Vec::new();
                if let Some(v) = since {
                    parts.push(format!("since {v}"));
                }
                if let Some(v) = removal_in {
                    parts.push(format!("removal in {v}"));
                }
                if let Some(n) = note {
                    parts.push(format!("\"{n}\""));
                }

                if !parts.is_empty() {
                    write!(f, " ({})", parts.join(", "))?;
                }
            }
        }

        // 2. Format the introduction version (if present)
        if let Some(introduced) = &self.introduced {
            write!(f, " (introduced in v{introduced})")?;
        }

        Ok(())
    }
}

impl DisplayOneLine for VersionAnnotation {}
