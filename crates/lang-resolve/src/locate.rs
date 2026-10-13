// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Function to locate microcad documents.

use microcad_lang_base::{MUCAD_EXTENSIONS, Url};
use miette::Diagnostic;
use thiserror::Error;

use crate::Manifest;

#[derive(Error, Diagnostic, Debug)]
pub enum LocateError {
    #[error("No µcad file found at: {path}")]
    #[diagnostic(
        code(microcad::locate::file_not_found),
        help(
            "Ensure the file exists or has a supported extension ({})",
            MUCAD_EXTENSIONS
                .iter()
                .map(|ext| format!(".{ext}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    )]
    FileNotFound { path: std::path::PathBuf },

    #[error("Invalid file URL path: {url}")]
    #[diagnostic(code(microcad::locate::invalid_file_url))]
    InvalidFileUrl { url: Url },

    #[error("Failed to convert path '{path}' to a valid file URL")]
    #[diagnostic(code(microcad::locate::url_conversion_failed))]
    UrlConversionFailed { path: std::path::PathBuf },

    #[error("Failed to find absolute path for: {path}")]
    #[diagnostic(code(microcad::locate::canonicalize_failed))]
    CanonicalizeFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Return `true` if given path has a valid microcad extension
pub fn is_microcad_file(path: impl AsRef<std::path::Path>) -> bool {
    let path = path.as_ref();
    path.is_file()
        && path
            .extension()
            .map(|ext| MUCAD_EXTENSIONS.iter().any(|extension| *extension == ext))
            .unwrap_or(false)
}

/// Retrieve actual path of µcad file, even if this path does not have an extension or is a folder.
///
/// It is agnostic about file extension and will always return the file with the extension that actually exists.
/// If the path is a directory, it will be checked if the directory contains a `mod` file with µcad extension.
///
/// - my/library/my_design.mu -> my/library/my_design.mu
/// - my/library/my_design -> my/library/my_design.mu # In case the extension of the existing file is `.mu`.
/// - my/library/my_design -> my/library/my_design/mod.mu # `my_design` is directory and a module.
///
/// Retrieve actual path of µcad file, even if this path does not have an extension or is a folder.
pub fn resolved_path(path: impl AsRef<std::path::Path>) -> Result<std::path::PathBuf, LocateError> {
    let path = path.as_ref();

    // If the path already has a supported extension, check if it exists.
    if is_microcad_file(path) {
        return Ok(path.to_path_buf());
    }

    // If not, try all supported extensions.
    for ext in MUCAD_EXTENSIONS {
        let mut with_ext = path.to_path_buf();
        with_ext.set_extension(ext);
        if with_ext.exists() {
            return Ok(with_ext);
        }
    }

    // If the path is a directory, look for a `mod` file with any supported extension.
    if path.is_dir() {
        for ext in MUCAD_EXTENSIONS {
            let mut mod_path = path.to_path_buf();
            mod_path.push(format!("mod.{ext}"));
            if mod_path.exists() {
                return Ok(mod_path);
            }
        }
    }

    Err(LocateError::FileNotFound {
        path: path.to_path_buf(),
    })
}

pub fn file_module_path(
    path: impl AsRef<std::path::Path>,
    module_name: impl AsRef<str>,
) -> Result<std::path::PathBuf, LocateError> {
    let mut path_no_ext = path.as_ref().to_path_buf();

    path_no_ext.set_extension("");
    if path_no_ext.file_name().unwrap().to_str().unwrap() == "lib" {
        path_no_ext = path_no_ext.parent().unwrap().to_path_buf();
    }

    resolved_path(path_no_ext.join(module_name.as_ref()))
}

/// The entry point library path from a directory, usually `lib.mu`.
pub fn lib_mu_path(path: impl AsRef<std::path::Path>) -> Result<std::path::PathBuf, LocateError> {
    let path = path.as_ref().to_path_buf();
    for ext in MUCAD_EXTENSIONS {
        let path = path.join(format!("lib.{ext}"));
        if path.exists() {
            return Ok(path);
        }
    }

    println!("{:?}", std::env::current_dir().unwrap());

    Err(LocateError::FileNotFound { path })
}

/// mu.toml path
pub fn mu_toml_path(path: impl AsRef<std::path::Path>) -> std::path::PathBuf {
    let path = path.as_ref();
    path.join(Manifest::MU_TOML)
}

/// Convert an input (e.g. from command line) into a valid and unique URL to be used for any source.
///
/// TODO: Move this to `driver`
pub fn to_url(input: impl AsRef<str>) -> Result<Url, LocateError> {
    let input = input.as_ref();

    // Try to parse as a formal URL (e.g., https://, file://, mcad://)
    match Url::parse(input) {
        Ok(url) => match url.scheme() {
            // If we have a file scheme, try to resolve the file.
            "file" => {
                let path = url
                    .to_file_path()
                    .map_err(|_| LocateError::InvalidFileUrl { url: url.clone() })?;
                let resolved = resolved_path(path)?;
                Url::from_file_path(&resolved)
                    .map_err(|_| LocateError::UrlConversionFailed { path: resolved })
            }
            _ => Ok(url),
        },

        Err(_) => {
            // 3. Fallback: Treat input as a raw filesystem path
            let local_path = std::path::PathBuf::from(input);
            let resolved = resolved_path(local_path)?;

            // Canonicalize to absolute path to ensure the URL is valid
            let absolute_path = std::fs::canonicalize(&resolved).map_err(|source| {
                LocateError::CanonicalizeFailed {
                    path: resolved.clone(),
                    source,
                }
            })?;

            Url::from_file_path(&absolute_path).map_err(|_| LocateError::UrlConversionFailed {
                path: absolute_path,
            })
        }
    }
}
