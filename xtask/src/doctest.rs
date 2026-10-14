// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_driver::prelude as mu;

/// Test mode
///
/// The code block headers can look like this:
///
/// *
#[derive(Debug)]
pub enum TestMode {
    /// A todo test. Will appear in the test list but the test won't run.
    ///
    /// - `mu,test?mode=todo`
    Todo,
    /// A test that simply parses the source code.
    ///
    /// - `mu,test?mode=parse`
    Parse,
    /// A test that compiles the model
    /// .
    /// - `mu,test?mode=compile`
    Compile,
    /// A test that compiles and renders into a file (default)
    ///
    /// - `mu,test`
    /// - `mu,test?mode=export&path=test.svg`
    /// - `mu,test?mode=export&path=test.svg&resolution=0.1mm`
    Export {
        /// Preferred render resolution.
        resolution: mu::RenderResolution,
        /// Optional output path (will use an auto generated path in case `None`)
        path: Option<std::path::PathBuf>,
    },
}

impl Default for TestMode {
    fn default() -> Self {
        Self::Export {
            resolution: Default::default(),
            path: None,
        }
    }
}

#[derive(Debug, Default)]
pub enum ExpectedTestResult {
    /// Test is supposed to be ok.
    ///
    /// `mu,test`
    /// `mu,test?result=ok`
    #[default]
    Ok,
    /// This test is supposed to file
    ///
    /// `mu,test?result=fail`
    Fail,
}

/// The status of a test after it has run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestStatus {
    /// The test succeeded as expected.
    Ok,
    /// The test has not been run.
    Todo,
    /// The test failed unexpectedly.
    Fail,
    /// The test failed as expected.
    FailOk,
    /// The test succeeded, but is expected to fail.
    OkFail,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            TestStatus::Ok => "ok",
            TestStatus::Todo => "todo",
            TestStatus::Fail => "fail",
            TestStatus::FailOk => "fail_ok",
            TestStatus::OkFail => "ok_fail",
        };
        write!(f, "{status}")
    }
}

impl TestStatus {
    /// Returns `false` if the test failed unexpectedly or succeeded but is expected to fail.
    pub fn is_success(&self) -> bool {
        matches!(self, TestStatus::Ok | TestStatus::Todo | TestStatus::FailOk)
    }
}

#[derive(Debug)]
pub struct TestResult {
    name: String,
    result: TestStatus,
    log_output: String,
}

/// Directory, into which tests export their models.
const OUTPUT_DIR: &str = "target/doctest";

/// Diagnostics are rendered without colors, because they appear in markdown.
const LOG_OPTIONS: mu::base::DiagRenderOptions = mu::base::DiagRenderOptions {
    color: false,
    unicode: false,
};

/// A panic while running a test, e.g. a `todo!()` in the compiler.
#[derive(Debug)]
pub struct Panic(String);

impl From<Box<dyn std::any::Any + Send>> for Panic {
    fn from(payload: Box<dyn std::any::Any + Send>) -> Self {
        // Panic messages are either a `String` or a `&str`.
        match payload.downcast::<String>() {
            Ok(message) => Self(*message),
            Err(payload) => Self(
                payload
                    .downcast_ref::<&str>()
                    .map(|message| message.to_string())
                    .unwrap_or_default(),
            ),
        }
    }
}

impl std::fmt::Display for Panic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Panic: {}", self.0)
    }
}

#[derive(Debug)]
pub struct TestCase {
    name: String,
    mode: TestMode,
    /// Expected test result
    result: ExpectedTestResult,
    source: mu::Source,
}

impl TestCase {
    /// Create a test case from a markdown code block in the markdown file at `path`.
    ///
    /// Each named µcad code block is a test case, e.g. `mu,circle?mode=compile&result=fail`.
    /// The test parameters are given by the header query, see [`TestMode`] and [`ExpectedTestResult`].
    /// Returns `None` for other code blocks and an error for invalid test parameters.
    pub fn from_code_block(
        path: &std::path::Path,
        code_block: &mu::md::CodeBlock,
    ) -> Result<Option<Self>, String> {
        let header = &code_block.header;
        let Some(name) = &header.name else {
            return Ok(None);
        };
        if !header
            .lang
            .as_deref()
            .is_some_and(|lang| mu::MUCAD_EXTENSIONS.contains(&lang))
        {
            return Ok(None);
        }

        // E.g. `intro::basics::circle` for code block `circle` in `intro/basics.md`.
        let name = format!(
            "{}::{name}",
            path.with_extension("").to_string_lossy().replace('/', "::")
        );
        let param = |key: &str| {
            header
                .query
                .get(&mu::Identifier::from(key))
                .cloned()
                .flatten()
        };
        let invalid = |key: &str, value: &str| format!("Invalid `{key}={value}` in test `{name}`");

        let mode = match param("mode").as_deref() {
            Some("todo") => TestMode::Todo,
            Some("parse") => TestMode::Parse,
            Some("compile") => TestMode::Compile,
            None | Some("export") => TestMode::Export {
                resolution: match param("resolution") {
                    Some(resolution) => microcad_driver::resolution_from_str(&resolution)
                        .map_err(|_| invalid("resolution", &resolution))?,
                    None => Default::default(),
                },
                path: param("path").map(std::path::PathBuf::from),
            },
            Some(mode) => return Err(invalid("mode", mode)),
        };

        let result = match param("result").as_deref() {
            None | Some("ok") => ExpectedTestResult::Ok,
            Some("fail") => ExpectedTestResult::Fail,
            Some(result) => return Err(invalid("result", result)),
        };

        Ok(Some(Self {
            name,
            mode,
            result,
            source: mu::Source::from(code_block.code.as_str()),
        }))
    }

    /// Run the test with `compiler` and compare the outcome with the expected result.
    ///
    /// A panic, e.g. a `todo!()` in the compiler, is returned as error.
    pub fn run(&self, compiler: &mut mu::Compiler) -> Result<TestResult, Panic> {
        if let TestMode::Todo = self.mode {
            return Ok(self.test_result(TestStatus::Todo, String::new()));
        }

        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.execute(compiler)))?;

        let status = match (&outcome, &self.result) {
            (Ok(_), ExpectedTestResult::Ok) => TestStatus::Ok,
            (Err(_), ExpectedTestResult::Ok) => TestStatus::Fail,
            (Err(_), ExpectedTestResult::Fail) => TestStatus::FailOk,
            (Ok(_), ExpectedTestResult::Fail) => TestStatus::OkFail,
        };
        let (Ok(log_output) | Err(log_output)) = outcome;
        Ok(self.test_result(status, log_output))
    }

    /// Execute the test and return its log: `Ok` if it succeeded, `Err` if it failed.
    fn execute(&self, compiler: &mut mu::Compiler) -> Result<String, String> {
        match &self.mode {
            TestMode::Todo => Ok(String::new()),
            TestMode::Parse => self.parse(),
            TestMode::Compile => {
                let (model, log) = self.compile(compiler)?;
                Ok(format!("{log}\n-- Model --\n{model}"))
            }
            TestMode::Export { resolution, path } => {
                let (model, log) = self.compile(compiler)?;
                match self.export(&model, resolution, path.as_deref()) {
                    Ok(path) => Ok(format!("{log}\nExported '{}'.", path.display())),
                    Err(err) => Err(format!("{log}\n{err:?}")),
                }
            }
        }
    }

    fn test_result(&self, result: TestStatus, log_output: String) -> TestResult {
        TestResult {
            name: self.name.clone(),
            result,
            log_output,
        }
    }

    /// Parse the source and return the diagnostics.
    fn parse(&self) -> Result<String, String> {
        let diags = match mu::parse(&self.source) {
            Ok((_, issues)) | Err(issues) => issues.into_diagnostics(),
        };
        let log = diags
            .render_to_string(&&self.source, &LOG_OPTIONS)
            .unwrap_or_default();
        match diags.has_errors() {
            true => Err(log),
            false => Ok(log),
        }
    }

    /// Compile the source and return the model and the diagnostics.
    ///
    /// Like in the CLI, a model with errors is a failure.
    fn compile(&self, compiler: &mut mu::Compiler) -> Result<(mu::ModelTree, String), String> {
        match compiler.compile(self.source.clone()) {
            Ok((model, diags)) if !diags.has_errors() => Ok((
                model,
                compiler.render_diag_to_string_with(&diags, &LOG_OPTIONS),
            )),
            Ok((_, diags)) | Err(diags) => {
                Err(compiler.render_diag_to_string_with(&diags, &LOG_OPTIONS))
            }
        }
    }

    /// Export the model into [`OUTPUT_DIR`] and return the path of the exported file.
    ///
    /// Without a `path`, the file is named after the test.
    fn export(
        &self,
        model: &mu::ModelTree,
        resolution: &mu::RenderResolution,
        path: Option<&std::path::Path>,
    ) -> mu::Result<std::path::PathBuf> {
        let output_dir = std::path::Path::new(OUTPUT_DIR);
        let input = output_dir.join(self.name.replace("::", "/"));
        let output = path.map(|path| output_dir.join(path));

        let exporter = mu::Exporter::new().with_resolution(resolution.clone());
        let target = exporter.output_path(model, &input, output.as_deref())?;
        if let Some(dir) = target.parent() {
            std::fs::create_dir_all(dir).map_err(|err| mu::report(&err.to_string()))?;
        }
        exporter.export(model, &input, Some(&target))
    }
}

#[derive(Debug)]
pub struct TestSuite {
    name: Option<String>,
    test_cases: Vec<TestCase>,
}

/// A test result list with a markdown output.
#[derive(Debug)]
pub struct TestResultList {
    name: Option<String>,
    results: Vec<TestResult>,
}

impl TestResultList {
    /// Returns `true` if no test failed unexpectedly.
    pub fn is_success(&self) -> bool {
        self.results.iter().all(|result| result.result.is_success())
    }

    /// The number of tests per status, e.g. `12 ok, 2 fail_ok, 1 todo`.
    pub fn summary(&self) -> String {
        use TestStatus::*;
        [Ok, FailOk, Todo, Fail, OkFail]
            .into_iter()
            .filter_map(|status| {
                let count = self
                    .results
                    .iter()
                    .filter(|result| result.result == status)
                    .count();
                (count > 0).then(|| format!("{count} {status}"))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// A page with the result of each test and the logs of the failed tests.
    pub fn to_md(&self) -> mu::md::Markdown {
        use mu::md::{
            Block, CodeBlock, CodeBlockHeader, Section,
            table_builder::{Column, TableBuilder},
        };

        let summary = Block::Paragraph(format!(
            "**{count}** tests have been run with µcad **{version}**: {summary}",
            count = self.results.len(),
            version = mu::MUCAD_VERSION,
            summary = self.summary(),
        ));
        let table = TableBuilder::new()
            .columns([Column::right("Result"), Column::left("Test")])
            .rows(
                self.results
                    .iter()
                    .map(|result| [result.result.to_string(), result.name.clone()]),
            );

        let mut md = mu::md::Markdown::page(
            self.name.clone().unwrap_or(String::from("Test result")),
            [summary, table.into()],
        );

        let failures: Vec<_> = self
            .results
            .iter()
            .filter(|result| !result.result.is_success())
            .collect();
        if !failures.is_empty() {
            md.add_section(Section::new(2, "Failures", []));
            failures.into_iter().for_each(|result| {
                let log = Block::CodeBlock(CodeBlock {
                    header: CodeBlockHeader {
                        lang: Some("text".into()),
                        name: None,
                        query: Default::default(),
                    },
                    code: result.log_output.trim_end().to_string(),
                    line_offset: 0,
                });
                md.add_section(Section::new(3, &result.name, [log]));
            });
        }

        md
    }
}

impl std::fmt::Display for TestResultList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_md())
    }
}

impl TestSuite {
    /// Create a new test from an mdbook.
    ///
    /// Fails, if a code block has invalid test parameters.
    pub fn from_mdbook(mdbook: &mu::md::MdBook) -> Result<Self, String> {
        let mut test_cases = mdbook
            .code_blocks()
            .filter_map(|(path, code_block)| {
                TestCase::from_code_block(&path, code_block).transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        // The markdown files are unordered, sort for a stable output.
        test_cases.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Self {
            name: mdbook.config.title().map(|title| title.to_string()),
            test_cases,
        })
    }

    /// Run all tests with a compiler for `config` and print the status of each test,
    /// like `cargo test`.
    pub fn run(&self, config: &mu::DriverConfig) -> TestResultList {
        let mut compiler = mu::Compiler::from_config(config);
        eprintln!("running {} tests", self.test_cases.len());

        let results = self
            .test_cases
            .iter()
            .map(|test_case| {
                let result = match test_case.run(&mut compiler) {
                    Ok(result) => result,
                    // A panic fails the test, but not the test suite.
                    Err(panic) => {
                        // The compiler may be broken after a panic, e.g. by a poisoned lock.
                        compiler = mu::Compiler::from_config(config);
                        test_case.test_result(TestStatus::Fail, panic.to_string())
                    }
                };
                eprintln!("test {} ... {}", result.name, result.result);
                result
            })
            .collect();

        let results = TestResultList {
            name: self.name.clone(),
            results,
        };

        // Like `cargo test`, show the output of the failed tests.
        let failures = results
            .results
            .iter()
            .filter(|result| !result.result.is_success());
        for (i, result) in failures.enumerate() {
            if i == 0 {
                eprintln!("\nfailures:");
            }
            eprintln!("\n---- {} ----\n{}", result.name, result.log_output);
        }

        eprintln!("\ntest result: {}", results.summary());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_case_from_header(header: &str) -> Result<Option<TestCase>, String> {
        let code_block = mu::md::CodeBlock {
            header: header.parse().expect("A code block header"),
            code: "__mu::geo2d::Circle(radius = 1mm);".into(),
            line_offset: 0,
        };
        TestCase::from_code_block(std::path::Path::new("intro/basics.md"), &code_block)
    }

    const CIRCLE: &str = "__mu::geo2d::Circle(radius = 1mm);";

    fn test_case(header: &str, code: &str) -> TestCase {
        let code_block = mu::md::CodeBlock {
            header: header.parse().expect("A code block header"),
            code: code.into(),
            line_offset: 0,
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

        let test_case =
            test_case_from_header("mu,circle?mode=export&path=circle.svg&resolution=0.5mm")
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
            run("mu,a?mode=todo", "this is no µcad").result,
            TestStatus::Todo
        );
        assert_eq!(run("mu,a?mode=parse", CIRCLE).result, TestStatus::Ok);

        let result = run("mu,a?mode=compile", CIRCLE);
        assert_eq!(result.result, TestStatus::Ok, "{}", result.log_output);
        assert!(result.log_output.contains("-- Model --"));

        let dir = tempfile::tempdir().expect("temp dir");
        let svg = dir.path().join("circle.svg");
        let result = run(&format!("mu,a?mode=export&path={}", svg.display()), CIRCLE);
        assert_eq!(result.result, TestStatus::Ok, "{}", result.log_output);
        assert!(svg.is_file());
    }

    #[test]
    fn run_expected_result() {
        let parse_error = "__mu::geo2d::Circle(radius = 1mm;";
        assert_eq!(run("mu,a?mode=parse", parse_error).result, TestStatus::Fail);
        assert_eq!(
            run("mu,a?mode=parse&result=fail", parse_error).result,
            TestStatus::FailOk
        );
        assert_eq!(
            run("mu,a?mode=compile&result=fail", "Unknown();").result,
            TestStatus::FailOk
        );
        assert_eq!(
            run("mu,a?mode=compile&result=fail", CIRCLE).result,
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
        .run(&no_std());
        assert!(results.is_success());
        assert_eq!(results.summary(), "1 ok, 1 fail_ok, 1 todo");

        let results = suite(vec![
            test_case("mu,a?mode=compile", CIRCLE),
            test_case("mu,b?mode=compile", "Unknown();"),
        ])
        .run(&no_std());
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
        .run(&no_std());
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
}
