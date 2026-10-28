// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Change the version of the whole workspace, e.g. `cargo xtask change-version 0.6.1`.
//!
//! The version is changed in the root `Cargo.toml`, which all crates inherit, in its internal
//! dependencies, in the manifests of the standard library and in the npm package of the web
//! viewer. The editor extensions have versions of their own, which are not changed.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use semver::Version;
use toml_edit::DocumentMut;

/// The manifests of µcad libraries with the version of the workspace, relative to its root.
const MU_TOMLS: &[&str] = &[
    "crates/std/lib/std/mu.toml",
    "crates/driver/tests/data/lib/std/mu.toml",
];

/// The npm files with the version of the workspace and the number of versions of their package
/// in them: the lock file has it twice.
const NPM_FILES: &[(&str, usize)] = &[
    ("editors/web-viewer/package.json", 1),
    ("editors/web-viewer/package-lock.json", 2),
];

/// The root of the workspace, the parent of the `xtask` crate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("The workspace of xtask")
        .to_path_buf()
}

/// Replace the value `version` keeping its formatting, e.g. the spaces around it.
fn set_version(version: &mut toml_edit::Value, new: &Version) {
    let decor = version.decor().clone();
    *version = new.to_string().into();
    *version.decor_mut() = decor;
}

/// The version of the workspace in the root `Cargo.toml`, and its text with `new` as version of
/// the workspace and of its internal dependencies, which have a path.
fn cargo_toml_with_version(text: &str, new: &Version) -> Result<(String, String), Box<dyn Error>> {
    let mut doc: DocumentMut = text.parse()?;
    let version = doc["workspace"]["package"]["version"]
        .as_value_mut()
        .ok_or("No version in [workspace.package]")?;
    let old = version
        .as_str()
        .ok_or("The version is no string")?
        .to_string();
    set_version(version, new);

    let dependencies = doc["workspace"]["dependencies"]
        .as_table_like_mut()
        .ok_or("No [workspace.dependencies]")?;
    for (_, dependency) in dependencies.iter_mut() {
        if let Some(dependency) = dependency.as_table_like_mut()
            && dependency.contains_key("path")
            && let Some(version) = dependency.get_mut("version").and_then(|v| v.as_value_mut())
        {
            set_version(version, new);
        }
    }
    Ok((old, doc.to_string()))
}

/// The version of a library manifest `mu.toml`, and its text with the version `new`.
fn mu_toml_with_version(text: &str, new: &Version) -> Result<(String, String), Box<dyn Error>> {
    let mut doc: DocumentMut = text.parse()?;
    let version = doc["library"]["version"]
        .as_value_mut()
        .ok_or("No version in [library]")?;
    let old = version
        .as_str()
        .ok_or("The version is no string")?
        .to_string();
    set_version(version, new);
    Ok((old, doc.to_string()))
}

/// The version of the package of an npm file, and its text with the version `new`.
///
/// The version of the package is the first `count` versions in the file, e.g. twice in a lock
/// file, before the versions of its dependencies, which do not change.
fn npm_with_version(
    text: &str,
    new: &Version,
    count: usize,
) -> Result<(String, String), Box<dyn Error>> {
    const KEY: &str = "\"version\": \"";
    // The spans of the versions of the package, without quotes.
    let spans: Vec<_> = (text.match_indices(KEY).take(count))
        .map(|(index, _)| {
            let start = index + KEY.len();
            text[start..].find('"').map(|length| start..start + length)
        })
        .collect::<Option<_>>()
        .ok_or("A version is not closed")?;
    let old = match spans.first() {
        Some(span) if spans.len() == count => text[span.clone()].to_string(),
        _ => return Err(format!("The package has not {count} versions").into()),
    };
    if spans.iter().any(|span| text[span.clone()] != old) {
        return Err("The versions of the package differ".into());
    }
    let mut changed = text.to_string();
    for span in spans.into_iter().rev() {
        changed.replace_range(span, &new.to_string());
    }
    Ok((old, changed))
}

/// Change the version of the workspace to `version` in all files, see the [module](self), and
/// update `Cargo.lock`. With `dry_run`, the changes are only listed.
///
/// The version is set in each file whatever its version, so that a version, which has been
/// changed in some files only, is completed. Fails without writing, if `version` is no version
/// or a file has no version.
pub fn change_version(version: &str, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let new = Version::parse(version).map_err(|err| format!("Invalid version {version}: {err}"))?;
    let root = workspace_root();
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).map_err(|err| format!("Cannot read {path}: {err}"))
    };
    // An error in the file `path`.
    fn in_file(path: &str) -> impl Fn(Box<dyn Error>) -> String + '_ {
        move |err| format!("{path}: {err}")
    }

    // The files with their old versions and their texts with the new version.
    let mut files = vec![(
        "Cargo.toml",
        cargo_toml_with_version(&read("Cargo.toml")?, &new).map_err(in_file("Cargo.toml"))?,
    )];
    for path in MU_TOMLS {
        files.push((
            path,
            mu_toml_with_version(&read(path)?, &new).map_err(in_file(path))?,
        ));
    }
    for (path, count) in NPM_FILES {
        files.push((
            path,
            npm_with_version(&read(path)?, &new, *count).map_err(in_file(path))?,
        ));
    }

    println!("Version {new}{}", if dry_run { " (dry run)" } else { "" });
    let mut changed = false;
    for (path, (old, text)) in &files {
        if *old == new.to_string() {
            println!("  {path}: {old}");
            continue;
        }
        changed = true;
        println!("  {path}: {old} -> {new}");
        if !dry_run {
            fs::write(root.join(path), text)
                .map_err(|err| format!("Cannot write {path}: {err}"))?;
        }
    }
    if dry_run {
        return Ok(());
    }
    if !changed {
        println!("The workspace has already the version {new}.");
    }

    let status = Command::new(env!("CARGO"))
        .args(["update", "--workspace", "--offline"])
        .current_dir(&root)
        .status()?;
    match status.success() {
        true => Ok(()),
        false => Err("Cannot update Cargo.lock: cargo update --workspace failed".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARGO_TOML: &str = r#"[workspace.package]
description = "µcad"
version = "0.6.0"  # the version
edition = "2024"

[workspace.dependencies]
mucad-core = { path = "crates/core", version = "0.6.0" }
mucad-hash = { path = "crates/hash", version   =   "0.5.0" }
serde = { version = "0.6.0", features = ["derive"] }
log = "0.4"
"#;

    fn version(text: &str) -> Version {
        Version::parse(text).expect("A version")
    }

    /// The version of the workspace and of its internal dependencies change, also from another
    /// version, the formatting is kept and external dependencies do not change.
    #[test]
    fn cargo_toml() {
        let (old, text) = cargo_toml_with_version(CARGO_TOML, &version("0.6.1")).expect("Changed");
        assert_eq!(old, "0.6.0");
        assert_eq!(
            text,
            r#"[workspace.package]
description = "µcad"
version = "0.6.1"  # the version
edition = "2024"

[workspace.dependencies]
mucad-core = { path = "crates/core", version = "0.6.1" }
mucad-hash = { path = "crates/hash", version   =   "0.6.1" }
serde = { version = "0.6.0", features = ["derive"] }
log = "0.4"
"#
        );
        // Without a version of the workspace.
        assert!(cargo_toml_with_version("[workspace]\n", &version("0.6.1")).is_err());
    }

    /// The version of a library manifest changes, the formatting is kept.
    #[test]
    fn mu_toml() {
        let text = "[library]\nname = \"std\"\nversion = \"0.6.0\"\nno_std = true # comment\n";
        let (old, changed) = mu_toml_with_version(text, &version("0.7.0-rc.1")).expect("Changed");
        assert_eq!(old, "0.6.0");
        assert_eq!(
            changed,
            "[library]\nname = \"std\"\nversion = \"0.7.0-rc.1\"\nno_std = true # comment\n"
        );
    }

    const LOCK: &str = r#"{
  "name": "mucad-web-viewer",
  "version": "0.6.0",
  "packages": {
    "": {
      "name": "mucad-web-viewer",
      "version": "0.6.0"
    },
    "node_modules/a": {
      "version": "0.6.0"
    }
  }
}
"#;

    /// Only the versions of the npm package change, not the ones of its dependencies.
    #[test]
    fn npm() {
        let (old, changed) = npm_with_version(LOCK, &version("0.6.1"), 2).expect("Changed");
        assert_eq!(old, "0.6.0");
        assert_eq!(changed, LOCK.replacen("0.6.0", "0.6.1", 2));
    }

    /// The versions of the package must be the same.
    #[test]
    fn npm_different_versions() {
        let lock = LOCK.replacen("0.6.0", "0.5.0", 1);
        assert!(npm_with_version(&lock, &version("0.6.1"), 2).is_err());
        assert!(npm_with_version("{}", &version("0.6.1"), 1).is_err());
    }
}
