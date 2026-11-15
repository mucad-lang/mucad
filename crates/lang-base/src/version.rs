// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// The version string from cargo.
pub const MUCAD_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use semver::Version;

use crate::DisplayOneLine;

/// Storage for µcad language version
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize)]
#[display("{_0}")]
pub struct LanguageVersion(Version);

impl LanguageVersion {
    /// The current µcad language version
    pub fn current() -> Self {
        Self(Version::parse(MUCAD_VERSION).expect("A valid cargo package version"))
    }

    /// Checks if a version string is compatible with the current µcad compiler
    pub fn is_compatible(&self) -> bool {
        use semver::VersionReq;

        // Use caret requirements: ^0.1.0 allows 0.1.x but not 0.2.0
        let req = VersionReq::parse(&format!("^{}", MUCAD_VERSION)).unwrap();

        req.matches(&self.0)
    }
}

impl From<Version> for LanguageVersion {
    fn from(version: Version) -> Self {
        Self(version)
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

/// Represents the stability level of a symbol in µcad, e.g. of a function, an initializer or a
/// property, which are part of the API of a library.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stability {
    /// `#[experimental]`: the symbol may change or be removed in any version.
    Experimental,
    /// `#[stable]`, the default.
    Stable,
    /// `#[deprecated]` or `#[deprecated(since = "0.5.0", note = "…", removal_in = "0.7.0")]`.
    Deprecated {
        /// The version since which the symbol is deprecated, without one in all versions.
        since: Option<Version>,
        /// A note, e.g. which symbol to use instead.
        note: Option<String>,
        /// The version in which the symbol will be removed.
        removal_in: Option<Version>,
    },
}

impl Stability {
    /// Whether the symbol is deprecated, e.g. by `#[deprecated]`.
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::Deprecated { .. })
    }

    /// When a deprecated symbol was deprecated and will be removed, e.g. `Deprecated since
    /// 0.5.0, will be removed in 0.7.0`, without its note, `None` if it is not deprecated.
    pub fn deprecation(&self) -> Option<String> {
        let Self::Deprecated {
            since, removal_in, ..
        } = self
        else {
            return None;
        };
        let mut text = String::from("Deprecated");
        if let Some(since) = since {
            text += &format!(" since {since}");
        }
        if let Some(removal_in) = removal_in {
            text += &format!(", will be removed in {removal_in}");
        }
        Some(text)
    }

    /// The note of a deprecated symbol, e.g. which symbol to use instead.
    pub fn deprecation_note(&self) -> Option<&str> {
        match self {
            Self::Deprecated { note, .. } => note.as_deref(),
            _ => None,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn version(version: &str) -> Version {
        Version::parse(version).expect("A valid version")
    }

    fn deprecated(since: Option<&str>, note: Option<&str>, removal_in: Option<&str>) -> Stability {
        Stability::Deprecated {
            since: since.map(version),
            note: note.map(str::to_string),
            removal_in: removal_in.map(version),
        }
    }

    fn annotation(introduced: Option<&str>, stability: Stability) -> VersionAnnotation {
        VersionAnnotation {
            introduced: introduced.map(version),
            stability,
        }
    }

    #[test]
    fn default_is_stable() {
        assert_eq!(VersionAnnotation::default(), VersionAnnotation::STABLE);
        assert_eq!(VersionAnnotation::default().stability, Stability::Stable);
        assert_eq!(VersionAnnotation::default().introduced, None);
    }

    #[test]
    fn available() {
        let always = annotation(None, Stability::Stable);
        assert!(always.is_available_at(&version("0.1.0")));

        let introduced = annotation(Some("0.6.0"), Stability::Experimental);
        assert!(!introduced.is_available_at(&version("0.5.9")));
        assert!(introduced.is_available_at(&version("0.6.0")));
        assert!(introduced.is_available_at(&version("1.0.0")));
    }

    #[test]
    fn deprecated_at() {
        let target = version("0.6.0");
        assert!(!annotation(None, Stability::Stable).is_deprecated_at(&target));
        assert!(!annotation(None, Stability::Experimental).is_deprecated_at(&target));

        // Without a version, the symbol is deprecated in all versions.
        assert!(annotation(None, deprecated(None, None, None)).is_deprecated_at(&target));

        let since = annotation(None, deprecated(Some("0.6.0"), None, None));
        assert!(!since.is_deprecated_at(&version("0.5.9")));
        assert!(since.is_deprecated_at(&target));
        assert!(since.is_deprecated_at(&version("0.7.0")));
    }

    #[test]
    fn deprecation() {
        assert_eq!(Stability::Stable.deprecation(), None);
        assert_eq!(Stability::Experimental.deprecation(), None);
        assert!(!Stability::Experimental.is_deprecated());

        let plain = deprecated(None, None, None);
        assert!(plain.is_deprecated());
        assert_eq!(plain.deprecation().as_deref(), Some("Deprecated"));
        assert_eq!(plain.deprecation_note(), None);

        let all = deprecated(Some("0.5.0"), Some("Use g()"), Some("0.7.0"));
        assert_eq!(
            all.deprecation().as_deref(),
            Some("Deprecated since 0.5.0, will be removed in 0.7.0")
        );
        assert_eq!(all.deprecation_note(), Some("Use g()"));
        assert_eq!(
            deprecated(None, None, Some("0.7.0"))
                .deprecation()
                .as_deref(),
            Some("Deprecated, will be removed in 0.7.0")
        );
    }

    #[test]
    fn display() {
        assert_eq!(annotation(None, Stability::Stable).to_string(), "stable");
        assert_eq!(
            annotation(Some("0.6.0"), Stability::Experimental).to_string(),
            "experimental (introduced in v0.6.0)"
        );
        assert_eq!(
            annotation(None, deprecated(None, None, None)).to_string(),
            "deprecated"
        );
        assert_eq!(
            annotation(None, deprecated(None, Some("Use g()"), None)).to_string(),
            "deprecated (\"Use g()\")"
        );
        assert_eq!(
            annotation(
                Some("0.4.0"),
                deprecated(Some("0.5.0"), Some("Use g()"), Some("0.7.0"))
            )
            .to_string(),
            "deprecated (since 0.5.0, removal in 0.7.0, \"Use g()\") (introduced in v0.4.0)"
        );
    }
}
