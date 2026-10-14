// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::Url;
use microcad_lang_resolve::locate;

#[test]
fn test_external_url_pass_through() {
    let https_url = "https://example.com/design.mcad";
    let result = locate::to_url(https_url).unwrap();
    assert_eq!(result.as_str(), https_url);
}

#[test]
fn test_filesystem_resolution_with_extension() {
    let dir = tempfile::tempdir().expect("No error");
    let file_path = dir.path().join("my_design.mu");
    std::fs::write(&file_path, "test content").unwrap();

    // Test raw string path
    let input = file_path.to_str().unwrap();
    let result = locate::to_url(input).unwrap();

    assert_eq!(result.scheme(), "file");
    // Canonicalize returns the absolute path, so we compare URLs
    assert_eq!(
        result,
        Url::from_file_path(std::fs::canonicalize(&file_path).unwrap()).unwrap()
    );
}

#[test]
fn test_filesystem_resolution_missing_extension() {
    let dir = tempfile::tempdir().expect("No error");
    let file_path = dir.path().join("logic.mcad"); // Using one of the extensions
    std::fs::write(&file_path, "test content").expect("No error");

    // Input WITHOUT extension
    let input = dir.path().join("logic").to_str().unwrap().to_string();
    let result = locate::to_url(&input).unwrap();

    assert!(result.as_str().ends_with("logic.mcad"));
}

#[test]
fn test_directory_mod_resolution() {
    let dir = tempfile::tempdir().expect("No error");
    let sub_dir = dir.path().join("my_module");
    std::fs::create_dir(&sub_dir).expect("No error");

    let mod_file = sub_dir.join("mod.mu");
    std::fs::write(&mod_file, "module content").expect("No error");

    // Input is the directory path
    let input = sub_dir.to_str().unwrap();
    let result = locate::to_url(input).unwrap();

    assert!(
        result.to_file_path().unwrap().ends_with("my_module/mod.mu"),
        "{result}"
    );
}

#[test]
fn test_invalid_path_fails() {
    let result = locate::to_url("/non/existent/path/at/all/ever");
    assert!(result.is_err());
}

#[test]
fn test_file_scheme_input() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("direct.mu");
    std::fs::write(&file_path, "content").unwrap();

    let file_url = Url::from_file_path(&file_path).unwrap();
    let result = locate::to_url(file_url.as_str()).unwrap();

    assert_eq!(result, file_url);
}

/// The root of a library is the directory with a `mu.toml` or the topmost source file.
#[test]
fn library_root() {
    let root = |path: &str| locate::library_root(format!("tests/test_cases/{path}"));
    let path = |path: &str| std::path::PathBuf::from(format!("tests/test_cases/{path}"));

    // File modules of a single file.
    assert_eq!(root("file_module/foo/bar.mu"), path("file_module.mu"));
    assert_eq!(root("file_module/sub/mod.mu"), path("file_module.mu"));
    assert_eq!(root("file_module.mu"), path("file_module.mu"));

    // Source files of a library with a manifest.
    assert_eq!(root("self_reference/geo.mu"), path("self_reference"));
    assert_eq!(root("self_reference/lib.mu"), path("self_reference"));

    // A single file is its own root.
    assert_eq!(root("resolve.mu"), path("resolve.mu"));
}
