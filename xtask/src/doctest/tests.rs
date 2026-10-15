// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

fn test_case_from_header(header: &str) -> Result<Option<TestCase>, String> {
    let code_block = mu::md::CodeBlock {
        header: header.parse().expect("A code block header"),
        code: "__mu::geo2d::Circle(radius = 1mm);".into(),
        line_offset: 0,
        path: None,
    };
    TestCase::from_code_block(std::path::Path::new("intro/basics.md"), &code_block)
}

const CIRCLE: &str = "__mu::geo2d::Circle(radius = 1mm);";

fn test_case(header: &str, code: &str) -> TestCase {
    let code_block = mu::md::CodeBlock {
        header: header.parse().expect("A code block header"),
        code: code.into(),
        line_offset: 0,
        path: None,
    };
    TestCase::from_code_block(std::path::Path::new("intro/basics.md"), &code_block)
        .expect("Valid")
        .expect("A test")
}

/// A configuration without standard library.
fn no_std() -> mu::DriverConfig {
    mu::DriverConfig {
        no_std: true,
        ..Default::default()
    }
}

/// A compiler without standard library.
fn compiler() -> mu::Compiler {
    mu::Compiler::from_config(&no_std())
}

/// Run a test of `code` with a compiler without standard library.
fn run(header: &str, code: &str) -> TestResult {
    test_case(header, code)
        .run(&mut compiler())
        .expect("No panic")
}

#[test]
fn from_code_block_defaults() {
    let test_case = test_case_from_header("mu,circle")
        .expect("Valid")
        .expect("A test");
    assert_eq!(test_case.name, "intro::basics::circle");
    assert!(matches!(
        test_case.mode,
        TestMode::Export { path: None, .. }
    ));
    assert!(matches!(test_case.result, ExpectedTestResult::Ok));
}

#[test]
fn from_code_block_parameters() {
    let test_case = test_case_from_header("mu,circle?mode=parse&result=fail")
        .expect("Valid")
        .expect("A test");
    assert!(matches!(test_case.mode, TestMode::Parse));
    assert!(matches!(test_case.result, ExpectedTestResult::Fail));

    let test_case = test_case_from_header("mu,circle?mode=export&path=circle.svg&resolution=0.5mm")
        .expect("Valid")
        .expect("A test");
    let TestMode::Export { resolution, path } = test_case.mode else {
        panic!("An export test");
    };
    assert!((resolution.linear - 0.5).abs() < 1e-6);
    assert_eq!(path, Some("circle.svg".into()));

    for (header, mode) in [("mu,a?mode=todo", "todo"), ("mu,a?mode=compile", "compile")] {
        let test_case = test_case_from_header(header)
            .expect("Valid")
            .expect("A test");
        assert!(
            matches!(
                (&test_case.mode, mode),
                (TestMode::Todo, "todo") | (TestMode::Compile, "compile")
            ),
            "{header}"
        );
    }
}

#[test]
fn from_code_block_no_test() {
    // Unnamed code blocks and other languages are no tests.
    assert!(test_case_from_header("mu").expect("Valid").is_none());
    assert!(
        test_case_from_header("rust,circle")
            .expect("Valid")
            .is_none()
    );
}

#[test]
fn from_code_block_invalid() {
    for header in [
        "mu,circle?mode=foo",
        "mu,circle?result=maybe",
        "mu,circle?resolution=fine",
    ] {
        let err = test_case_from_header(header).expect_err(header);
        assert!(err.contains("intro::basics::circle"), "{err}");
    }
}

#[test]
fn run_modes() {
    assert_eq!(
        run("mu,a?mode=todo", "this is no µcad").status,
        TestStatus::Todo
    );
    assert_eq!(run("mu,a?mode=parse", CIRCLE).status, TestStatus::Ok);

    let result = run("mu,a?mode=compile", CIRCLE);
    assert_eq!(result.status, TestStatus::Ok, "{}", result.log_output);
    assert!(result.log_output.contains("-- Model --"));

    let dir = tempfile::tempdir().expect("temp dir");
    let svg = dir.path().join("circle.svg");
    let result = run(&format!("mu,a?mode=export&path={}", svg.display()), CIRCLE);
    assert_eq!(result.status, TestStatus::Ok, "{}", result.log_output);
    assert!(svg.is_file());
}

/// An export test of code without geometry is only compiled and passes.
#[test]
fn run_export_without_geometry() {
    let result = run("mu,a", "a = 2cm;\nb = a * 2;");
    assert_eq!(result.status, TestStatus::Ok, "{}", result.log_output);
    assert!(result.log_output.contains("No model to export."));
}

/// An export test of a sketch without primitives is only compiled and passes.
#[test]
fn run_export_empty_sketch() {
    let result = run(
        "mu,a",
        "sketch S() {\n    __mu::debug::assert(cond = true);\n}\nS();",
    );
    assert_eq!(result.status, TestStatus::Ok, "{}", result.log_output);
    assert!(result.log_output.contains("No model to export."));
}

#[test]
fn run_expected_result() {
    let parse_error = "__mu::geo2d::Circle(radius = 1mm;";
    assert_eq!(run("mu,a?mode=parse", parse_error).status, TestStatus::Fail);
    assert_eq!(
        run("mu,a?mode=parse&result=fail", parse_error).status,
        TestStatus::FailOk
    );
    assert_eq!(
        run("mu,a?mode=compile&result=fail", "Unknown();").status,
        TestStatus::FailOk
    );
    assert_eq!(
        run("mu,a?mode=compile&result=fail", CIRCLE).status,
        TestStatus::OkFail
    );
}

#[test]
fn run_suite() {
    let suite = |test_cases| TestSuite {
        name: None,
        test_cases,
    };

    let results = suite(vec![
        test_case("mu,a?mode=compile", CIRCLE),
        test_case("mu,b?mode=compile&result=fail", "Unknown();"),
        test_case("mu,c?mode=todo", ""),
    ])
    .run(true, &no_std());
    assert!(results.is_success());
    assert_eq!(results.summary(), "1 ok, 1 fail_ok, 1 todo");

    let results = suite(vec![
        test_case("mu,a?mode=compile", CIRCLE),
        test_case("mu,b?mode=compile", "Unknown();"),
    ])
    .run(true, &no_std());
    assert!(!results.is_success());
    assert_eq!(results.summary(), "1 ok, 1 fail");
}

#[test]
fn panic_message() {
    let panic = |f: fn()| Panic::from(std::panic::catch_unwind(f).expect_err("A panic"));
    assert_eq!(panic(|| panic!("static")).to_string(), "Panic: static");
    assert_eq!(
        panic(|| panic!("formatted {}", 42)).to_string(),
        "Panic: formatted 42"
    );
}

#[test]
fn results_to_md() {
    let results = TestSuite {
        name: Some("Book".into()),
        test_cases: vec![
            test_case("mu,a?mode=compile", CIRCLE),
            test_case("mu,b?mode=compile", "Unknown();"),
        ],
    }
    .run(true, &no_std());
    let md = results.to_md().to_string();

    assert!(md.starts_with("# Book"), "{md}");
    assert!(md.contains("**2** tests have been run"), "{md}");
    assert!(md.contains("1 ok, 1 fail"), "{md}");
    assert!(md.contains("| intro::basics::a |"), "{md}");
    assert!(md.contains("## Failures"), "{md}");
    assert!(md.contains("### intro::basics::b"), "{md}");
    assert!(md.contains("Unresolved symbol: Unknown"), "{md}");
    // Logs are written without colors.
    assert!(!md.contains('\u{1b}'), "{md}");
}

#[test]
fn banner() {
    assert_eq!(
        test_case("mu,circle", CIRCLE).banner(),
        "[![test](.test/circle.svg)](.test/circle.log)"
    );
}

#[test]
fn set_banners() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("book.toml"), "[book]\ntitle = \"Test\"\n").expect("toml");
    std::fs::create_dir(dir.path().join("src")).expect("src");
    let md = "# Test\n\n```mu,circle\nCircle(r = 1mm);\n```\n\n```mu\nno test\n```\n";
    std::fs::write(dir.path().join("src/test.md"), md).expect("test.md");
    let mut mdbook = mu::md::MdBook::load(dir.path()).expect("A book");

    // The banner is inserted before the test, not before the unnamed code block.
    assert_eq!(
        TestSuite::set_banners(&mut mdbook),
        [std::path::PathBuf::from("test.md")]
    );
    let text = mdbook.md_files[std::path::Path::new("test.md")].to_string();
    assert_eq!(text.matches("[![test]").count(), 1, "{text}");
    assert!(
        text.contains("[![test](.test/circle.svg)](.test/circle.log)\n\n```mu,circle"),
        "{text}"
    );

    // The test sources have their location.
    let suite = TestSuite::from_mdbook(&mdbook).expect("A test suite");
    assert!(suite.test_cases[0].source.path().is_some());

    assert!(TestSuite::set_banners(&mut mdbook).is_empty());
}

#[test]
fn write_result_files() {
    let dir = tempfile::tempdir().expect("temp dir");
    let md_file = dir.path().join("test.md");
    let code_block = mu::md::CodeBlock {
        header: "mu,circle?mode=compile"
            .parse()
            .expect("A code block header"),
        code: CIRCLE.into(),
        line_offset: 4,
        path: Some(md_file.clone()),
    };
    let test_case = TestCase::from_code_block(std::path::Path::new("test.md"), &code_block)
        .expect("Valid")
        .expect("A test");

    TestSuite {
        name: None,
        test_cases: vec![test_case],
    }
    .run(false, &no_std());

    let banner = std::fs::read(dir.path().join(".test/circle.svg")).expect("A banner image");
    let ok_image =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("images/ok.svg"))
            .expect("The ok image");
    assert_eq!(banner, ok_image);

    let log = std::fs::read_to_string(dir.path().join(".test/circle.log")).expect("A log");
    assert!(log.contains("Test name: test::circle"), "{log}");
    assert!(log.contains("Expected result: ok"), "{log}");
    assert!(log.contains("test.md:5"), "{log}");
    assert!(log.contains(&format!("    5:   {CIRCLE}")), "{log}");
    assert!(log.contains("-- Model --"), "{log}");
    assert!(log.ends_with("-- Test Result --\nOK\n"), "{log}");
}
