// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Name, Url};
use serde::Serialize;

/// Kind of the source file
#[derive(Debug, Clone, Hash, Serialize, PartialEq, Eq, derive_more::From)]
pub enum SourceKind {
    /// A source url
    Url(Url),
    /// Source from a file path
    Path(std::path::PathBuf),
    /// A source from stdin
    Stdin,
    /// A virtual source
    Virtual(String),
    /// Raw &str
    Str,
}

/// The `file://` URL of a path.
#[cfg(any(unix, windows, target_os = "wasi"))]
fn url_from_file_path(path: &std::path::Path) -> Option<Url> {
    Url::from_file_path(path).ok()
}

/// The `file://` URL of a path, without a file system, e.g. in a browser.
#[cfg(not(any(unix, windows, target_os = "wasi")))]
fn url_from_file_path(path: &std::path::Path) -> Option<Url> {
    Url::parse(&format!("file://{}", path.display())).ok()
}

/// The path of a `file://` URL.
#[cfg(any(unix, windows, target_os = "wasi"))]
fn url_to_file_path(url: &Url) -> Option<std::path::PathBuf> {
    url.to_file_path().ok()
}

/// The path of a `file://` URL, without a file system, e.g. in a browser.
#[cfg(not(any(unix, windows, target_os = "wasi")))]
fn url_to_file_path(url: &Url) -> Option<std::path::PathBuf> {
    Some(std::path::PathBuf::from(url.path()))
}

impl SourceKind {
    /// Returns a fallback or explicit URL representation of the source.
    pub fn url(&self) -> Url {
        match self {
            SourceKind::Url(url) => url.clone(),
            SourceKind::Path(path) => {
                // Safely convert a local path to a file:// URL
                url_from_file_path(path)
                    .unwrap_or_else(|| Url::parse("file:///invalid-path").unwrap())
            }
            SourceKind::Stdin => Url::parse("stdin://-").unwrap(),
            SourceKind::Virtual(name) => {
                // URL-encode or format the virtual name into a schema
                let scheme = format!("virtual://{}", name);
                Url::parse(&scheme).unwrap_or_else(|_| Url::parse("virtual://unknown").unwrap())
            }
            SourceKind::Str => Url::parse("str:///").unwrap(),
        }
    }

    /// Returns a reference to the underlying path if this source lives on disk.
    pub fn path(&self) -> Option<std::path::PathBuf> {
        match self {
            SourceKind::Path(path) => Some(path.clone()),
            SourceKind::Url(url) => {
                // If it's a file:// URL, we can extract the path dynamically
                if url.scheme() == "file" {
                    url_to_file_path(url)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Return the relative file path from current directory.
    ///
    /// Without a current directory (e.g. in a browser) or a local file, the path is returned.
    pub fn relative_path(&self) -> Option<std::path::PathBuf> {
        self.path().map(
            |path| match (std::env::current_dir(), path.canonicalize()) {
                (Ok(current_dir), Ok(path)) => {
                    pathdiff::diff_paths(path, current_dir).unwrap_or_default()
                }
                _ => path.to_path_buf(),
            },
        )
    }

    /// Helper to identify if the resource exists on disk.
    pub fn is_local(&self) -> bool {
        self.path().is_some()
    }

    /// The relative path of the source file to displayed
    pub fn display_name(&self) -> String {
        self.relative_path()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(self.url().path().to_string())
    }

    /// Extracts "bar" from "foo/bar.mu")
    pub fn file_module_name(&self) -> Option<Name> {
        let path = self
            .relative_path()
            .unwrap_or_else(|| std::path::PathBuf::from(self.url().path()));
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            // If we have a `mod.mu` file, we need to take the parent as file module name.
            Some("mod") => path
                .parent()
                .and_then(|path| path.file_stem())
                .and_then(|s| s.to_str()),
            Some(s) => Some(s),
            None => None,
        };
        stem.map(|s| Name::from(s.to_string()))
    }
}

/// Represents *where* the code came from.
/// Stored once in the central SourceMap, indexed by SourceId.
#[derive(Debug, Clone, Hash, Serialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub kind: SourceKind,
    pub line_offset: Option<u32>,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 1. Get the path string or URL string via your existing helper
        let base = self.kind.display_name();

        // 2. Format with or without the suffix based on line_offset presence
        if let Some(offset) = self.line_offset {
            write!(f, "{}:{}", base, offset)
        } else {
            write!(f, "{}", base)
        }
    }
}

impl SourceLocation {
    /// New source location with specific source kind
    pub fn new(kind: impl Into<SourceKind>) -> Self {
        Self {
            kind: kind.into(),
            line_offset: None,
        }
    }

    /// New source location with a line offset
    pub fn with_line_offset(self, line_offset: u32) -> Self {
        Self {
            kind: self.kind,
            line_offset: Some(line_offset),
        }
    }

    /// Forwards to the underlying `SourceKind::url`
    #[inline]
    pub fn url(&self) -> Url {
        self.kind.url()
    }

    /// Forwards to the underlying `SourceKind::path`
    #[inline]
    pub fn path(&self) -> Option<std::path::PathBuf> {
        self.kind.path()
    }
}

impl From<SourceKind> for SourceLocation {
    fn from(kind: SourceKind) -> Self {
        Self::new(kind)
    }
}

impl From<std::path::PathBuf> for SourceLocation {
    fn from(path: std::path::PathBuf) -> Self {
        Self::new(path)
    }
}

impl From<Url> for SourceLocation {
    fn from(url: Url) -> Self {
        Self::new(url)
    }
}
