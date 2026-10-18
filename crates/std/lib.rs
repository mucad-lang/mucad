// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI install command.

use rust_embed::RustEmbed;
use thiserror::Error;

use mucad_lang_base::{LibraryId, LibraryInfo, VirtualFileSystem};

/// Standard library error.
#[derive(Debug, Error)]
pub enum StdLibError {
    /// Error during install or uninstall.
    #[error("An error during installation: {0}")]
    InstallError(#[from] std::io::Error),
}

/// The µcad standard library asset.
#[derive(RustEmbed)]
#[folder = "lib/std"]
pub struct StdLibEmbedded;

/// An instance of the standard library.
pub struct StdLib {
    /// Path of the library which `manifest.toml`.
    pub path: std::path::PathBuf,
}

impl StdLib {
    /// Try to reinstall into default path.
    pub fn reinstall(force: bool) -> Result<(), StdLibError> {
        let path = Self::default_path();
        if force {
            Self::uninstall(&path)?;
        }

        Self::install(path)
    }

    /// Get info for standard library
    pub fn info() -> LibraryInfo {
        LibraryInfo::new("std", crate::version())
    }

    /// Get standard library id.
    pub fn id() -> LibraryId {
        Self::info().id()
    }

    /// Install the standard library into the standard library path and return its manifest.
    pub fn install(path: impl AsRef<std::path::Path>) -> Result<(), StdLibError> {
        let path = path.as_ref();
        eprintln!(
            "Installing µcad standard library {} into {:?}...",
            crate::version(),
            path
        );

        std::fs::create_dir_all(path)?;

        // Extract all embedded files.
        StdLibEmbedded::iter().try_for_each(|file| {
            let file_path = path.join(file.as_ref());
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(
                file_path,
                StdLibEmbedded::get(file.as_ref())
                    .expect("embedded folder 'lib/std'")
                    .data,
            )
        })?;

        eprintln!("Successfully installed µcad standard library.");
        Ok(())
    }

    /// A virtual file system with the standard library in the library search path `search_path`,
    /// e.g. in a browser without a local file system.
    pub fn virtual_file_system(search_path: impl AsRef<std::path::Path>) -> VirtualFileSystem {
        let path = search_path.as_ref().join(Self::info().path());
        StdLibEmbedded::iter().fold(VirtualFileSystem::new(), |file_system, file| {
            let content = StdLibEmbedded::get(file.as_ref()).expect("embedded folder 'lib/std'");
            file_system.with_file(
                path.join(file.as_ref()),
                String::from_utf8_lossy(&content.data),
            )
        })
    }

    /// Uninstall the standard library from the standard library path.
    fn uninstall(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            eprintln!(
                "µcad standard library not found in {:?}. Nothing to uninstall.",
                path
            );
            return Ok(());
        }

        eprintln!("Removing µcad standard library from {:?}...", path);

        // Extract all embedded files.
        StdLibEmbedded::iter().try_for_each(|file| {
            let file_path = path.join(file.as_ref());
            std::fs::remove_file(file_path)
        })?;

        // All standard library files should been remove now.
        std::fs::remove_dir(path)?;

        eprintln!("Successfully uninstalled µcad standard library.");

        Ok(())
    }

    /// Global library search path + `./std`.
    pub fn default_path() -> std::path::PathBuf {
        global_library_search_path().join(Self::info().path())
    }
}

/// The global library search path.
///
/// Use the config directory in home directory `.config/mucad` in Release mode.
/// In Debug mode, the local directory `./crates/std/lib` is used.
pub fn global_library_search_path() -> std::path::PathBuf {
    #[cfg(not(debug_assertions))]
    return dirs::config_dir()
        .expect("config directory")
        .join("mucad")
        .join("lib");

    #[cfg(debug_assertions)]
    return std::path::PathBuf::from("./crates/std/lib");
}

/// Return the version number of this crate.
pub fn version() -> semver::Version {
    use std::str::FromStr;
    semver::Version::from_str(env!("CARGO_PKG_VERSION")).expect("Valid version")
}
