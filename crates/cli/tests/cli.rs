// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests of the CLI commands `check`, `compile` and `format`.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use mucad_driver::prelude as mu;
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

/// The `mucad` binary.
///
/// In debug builds, the standard library is searched relative to the working directory
/// (`./crates/std/lib`), so the binary runs in the workspace root.
fn mucad() -> Command {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("mucad"));
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

    mucad()
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

    mucad()
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

    mucad()
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

    mucad()
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

    mucad()
        .arg("check")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unclosed function arguments"))
        // The error is shown with its location in the source code.
        .stderr(predicate::str::contains("parse_error.mu:1:19"))
        .stderr(predicate::str::contains(PARSE_ERROR.trim_end()));
}

#[test]
fn check_missing_file() {
    let dir = tempfile::tempdir().expect("temp dir");

    mucad()
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

    mucad().arg("compile").arg(&input).assert().success();

    let files = std::fs::read_dir(dir.path()).expect("read dir").count();
    assert_eq!(files, 1, "only the source file is expected");
}

#[test]
fn compile_emits_artifacts() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    mucad()
        .arg("compile")
        .arg(&input)
        .args(["--emit-ast", "--emit-ir", "--emit-model"])
        .assert()
        .success();

    use mu::ArtifactKind::*;
    for kind in [Ast, Ir, ModelTree] {
        let artifact = dir.path().join(format!("valid.{ext}", ext = kind.ext()));
        assert!(artifact.is_file(), "{artifact:?} is missing");
    }
}

#[test]
fn compile_emits_model_to_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);
    let output = dir.path().join("out");

    mucad()
        .arg("compile")
        .arg(&input)
        .arg("--emit-model")
        .arg(&output)
        .assert()
        .success();

    assert!(dir.path().join("out.mumodel").is_file());
    assert!(!dir.path().join("valid.mumodel").exists());
}

#[test]
fn compile_parse_error_emits_no_model() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "parse_error.mu", PARSE_ERROR);

    mucad()
        .arg("compile")
        .arg(&input)
        .arg("--emit-model")
        .assert()
        .stderr(predicate::str::contains("cannot emit model"));

    assert!(!dir.path().join("parse_error.mumodel").exists());
}

#[test]
fn format_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "unformatted.mu", UNFORMATTED);

    mucad()
        .arg("format")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Formatted document."));

    assert_eq!(read(&input).trim_end(), FORMATTED);

    mucad()
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

    mucad().arg("fmt").arg(&input).assert().success();

    assert_eq!(read(&input).trim_end(), FORMATTED);
}

#[test]
fn format_stdin() {
    mucad()
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

    mucad()
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

    mucad()
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

    mucad()
        .arg("export")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("has errors"));

    let files = std::fs::read_dir(dir.path()).expect("read dir").count();
    assert_eq!(files, 1, "only the source file is expected");
}

#[test]
fn export_dry_run() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    mucad()
        .arg("export")
        .arg("--dry-run")
        .arg(&input)
        .assert()
        .success()
        .stderr(predicate::str::contains("valid.svg"));

    assert!(!dir.path().join("valid.svg").exists());
}

#[test]
fn export_resolution() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    mucad()
        .arg("export")
        .args(["--resolution", "0.05mm"])
        .arg(&input)
        .assert()
        .success();

    mucad()
        .arg("export")
        .args(["--resolution", "2"])
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid resolution"));
}

/// The artifacts of a source file are displayed in the order of the compiler stages.
#[test]
fn compile_displays_artifacts() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "valid.mu", VALID);

    mucad()
        .arg("compile")
        .arg(&input)
        .args(["--display", "model"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("Group"));

    mucad()
        .arg("compile")
        .arg(&input)
        .args(["--display", "model,ast,lib,ir"])
        .assert()
        .success()
        .stdout(
            predicate::str::is_match("(?s)--- AST ---.*--- IR ---.*--- Library ---.*--- Model ---")
                .expect("A regex"),
        );
}

/// A library has no AST, IR and model to display, only its symbols.
/// The library of the tests is not written.
#[test]
fn compile_displays_library() {
    let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("../driver/tests/data/lib/std");

    mucad()
        .arg("compile")
        .arg(&library)
        .args(["--display", "library,model", "--no-emit-lib"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--- Library ---"))
        .stdout(predicate::str::contains("Circle: Workbench"))
        .stderr(predicate::str::contains(
            "Cannot display Model: only available for source files",
        ));
}

/// Without output, a source is exported into the files of its export attributes.
#[test]
fn export_attributes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(
        &dir,
        "logo.mu",
        "#[export]\nshape = std::geo2d::Circle(radius = 1mm);\n\
         #[export = \"disc.stl\"]\nshape.std::ops::extrude(height = 1mm);\n",
    );

    mucad()
        .arg("export")
        .arg(&input)
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("logo__shape.svg"))
        .stderr(predicate::str::contains("disc.stl"));
    assert!(!dir.path().join("disc.stl").exists());

    mucad().arg("export").arg(&input).assert().success();
    assert!(dir.path().join("logo__shape.svg").is_file());
    assert!(dir.path().join("disc.stl").is_file());
    assert!(!dir.path().join("logo.svg").exists());
}

/// A library directory with `mu.toml` and `lib.mu`, without standard library.
fn library_dir(code: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    source_file(
        &dir,
        "mu.toml",
        "[library]\nname = \"shapes\"\nversion = \"0.1.0\"\nno_std = true\n",
    );
    source_file(&dir, "lib.mu", code);
    dir
}

/// A library is compiled into `<name>.mulib` in its directory, which can be loaded again.
#[test]
fn compile_writes_library() {
    let dir = library_dir("pub const SIZE = 10mm;");
    mucad().arg("compile").arg(dir.path()).assert().success();

    use mu::Artifact;
    let bytes = std::fs::read(dir.path().join("shapes.mulib")).expect("A library file");
    let lib = mu::Library::from_binary(&bytes).expect("A library");
    assert_eq!(lib.name().map(|name| name.as_str()), Some("shapes"));
}

/// With `--no-emit-lib`, a library is only compiled.
#[test]
fn compile_no_emit_lib() {
    let dir = library_dir("pub const SIZE = 10mm;");
    mucad()
        .arg("compile")
        .arg(dir.path())
        .arg("--no-emit-lib")
        .assert()
        .success();
    assert!(!dir.path().join("shapes.mulib").exists());
}

/// A library with errors is not written, e.g. with a parse error or an unresolved symbol.
#[test]
fn compile_library_with_errors() {
    for code in ["pub const SIZE = ;", "pub const SIZE = UNKNOWN;"] {
        let dir = library_dir(code);
        mucad()
            .arg("compile")
            .arg(dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("Library has issues"));
        assert!(!dir.path().join("shapes.mulib").exists(), "{code}");
    }
}

/// A source file is shared as link to the web viewer, with its QR code below.
#[test]
fn compile_displays_link() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = source_file(&dir, "disc.mu", "Circle(r = 10mm);\n");

    mucad()
        .arg("compile")
        .arg(&input)
        .args(["--display", "link,qrcode"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "--- Link ---\n{}Q2lyY2xlKHIgPSAxMG1tKTsK\n--- QR code ---\n",
            mu::share::PLAY_URL
        )))
        .stdout(predicate::str::contains('█'));
}

/// A source file with more than 2048 bytes is too long for a link.
#[test]
fn compile_link_too_long() {
    let dir = tempfile::tempdir().expect("temp dir");
    let code = format!("// {}\nCircle(r = 10mm);\n", "x".repeat(2048));
    let input = source_file(&dir, "long.mu", &code);

    mucad()
        .arg("compile")
        .arg(&input)
        .args(["--display", "link"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://").not())
        .stderr(predicate::str::contains(
            "Cannot display Link: the code has 2070 bytes, a link can contain at most 2048 bytes",
        ));
}
