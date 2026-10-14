// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests of the CLI commands `check`, `compile` and `format`.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

/// A valid source using the standard library.
const VALID: &str = "std::geo2d::Circle(radius = 1mm);\n";

/// A source with a resolve error.
const UNRESOLVED: &str = "Unknown(radius = 1mm);\n";

/// A source with a parse error.
const PARSE_ERROR: &str = "std::geo2d::Circle(radius = 1mm;\n";

/// A valid source which is not formatted.
const UNFORMATTED: &str = "std::geo2d::Circle( radius=1mm );\n";

/// The formatted version of [`UNFORMATTED`].
const FORMATTED: &str = "std::geo2d::Circle(radius = 1mm);";

/// The `microcad` binary.
///
/// In debug builds, the standard library is searched relative to the working directory
/// (`./crates/std/lib`), so the binary runs in the workspace root.
fn microcad() -> Command {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("microcad"));
    cmd.current_dir(workspace_root);
    cmd
}

/// Write a source into a temporary directory and return its path.
fn source_file(dir: &tempfile::TempDir, name: &str, code: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, code).expect("write source file");
    path
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read file")
}

#[test]
fn check_valid() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    microcad()
        .arg("check")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("is valid."));
}

#[test]
fn check_glob_import() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(
        &dir,
        "glob.mu",
        "use std::geo2d::*;\nCircle(radius = 1mm);\n",
    );

    microcad()
        .arg("check")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("is valid."));
}

/// The example of the CLI crate.
#[test]
fn check_example() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/hello.mu");

    microcad()
        .arg("check")
        .arg(&example)
        .assert()
        .success()
        .stderr(predicate::str::contains("is valid."));
}

#[test]
fn check_unresolved() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "unresolved.mu", UNRESOLVED);

    microcad()
        .arg("check")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unresolved symbol: Unknown"))
        .stderr(predicate::str::contains("has errors"));
}

#[test]
fn check_parse_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "parse_error.mu", PARSE_ERROR);

    microcad()
        .arg("check")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unclosed function arguments"))
        // The error is shown with its location in the source code.
        .stderr(predicate::str::contains("parse_error.mu:1:32"))
        .stderr(predicate::str::contains(PARSE_ERROR.trim_end()));
}

#[test]
fn check_missing_file() {
    let dir = tempfile::tempdir().expect("temp dir");

    microcad()
        .arg("check")
        .arg(dir.path().join("missing.mu"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn compile_emits_nothing_by_default() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    microcad().arg("compile").arg(&input).assert().success();

    let files = std::fs::read_dir(dir.path()).expect("read dir").count();
    assert_eq!(files, 1, "only the source file is expected");
}

#[test]
fn compile_emits_artifacts() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    microcad()
        .arg("compile")
        .arg(&input)
        .args(["--emit-ast", "--emit-ir", "--emit-model"])
        .assert()
        .success();

    for ext in ["ast", "ir", "model"] {
        let artifact = dir.path().join(format!("valid.mu.{ext}"));
        assert!(artifact.is_file(), "{artifact:?} is missing");
    }
}

#[test]
fn compile_emits_model_to_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);
    let output = dir.path().join("out");

    microcad()
        .arg("compile")
        .arg(&input)
        .arg("--emit-model")
        .arg(&output)
        .assert()
        .success();

    assert!(dir.path().join("out.model").is_file());
    assert!(!dir.path().join("valid.mu.model").exists());
}

#[test]
fn compile_parse_error_emits_no_model() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "parse_error.mu", PARSE_ERROR);

    microcad()
        .arg("compile")
        .arg(&input)
        .arg("--emit-model")
        .assert()
        .stderr(predicate::str::contains("cannot emit model"));

    assert!(!dir.path().join("parse_error.mu.model").exists());
}

#[test]
fn format_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "unformatted.mu", UNFORMATTED);

    microcad()
        .arg("format")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Formatted document."));

    assert_eq!(read(&input).trim_end(), FORMATTED);

    microcad()
        .arg("format")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("already formatted"));
}

#[test]
fn format_alias() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "unformatted.mu", UNFORMATTED);

    microcad().arg("fmt").arg(&input).assert().success();

    assert_eq!(read(&input).trim_end(), FORMATTED);
}

#[test]
fn format_stdin() {
    microcad()
        .arg("format")
        .write_stdin(UNFORMATTED)
        .assert()
        .success()
        .stdout(predicate::str::starts_with(FORMATTED));
}

#[test]
fn format_parse_error_keeps_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "parse_error.mu", PARSE_ERROR);

    microcad()
        .arg("format")
        .arg(&input)
        .assert()
        .stderr(predicate::str::contains("Parse error"));

    assert_eq!(read(&input), PARSE_ERROR);
}

#[test]
fn export_default_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    microcad()
        .arg("export")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Exported"));

    assert!(dir.path().join("valid.svg").is_file());
}

#[test]
fn export_errors_exports_nothing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "unresolved.mu", UNRESOLVED);

    microcad()
        .arg("export")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing exported"));

    let files = std::fs::read_dir(dir.path()).expect("read dir").count();
    assert_eq!(files, 1, "only the source file is expected");
}

#[test]
fn export_dry_run() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    microcad()
        .arg("export")
        .arg("--dry-run")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("valid.svg"));

    assert!(!dir.path().join("valid.svg").exists());
}
