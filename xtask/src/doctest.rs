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

#[derive(Debug)]
pub struct TestResult {
    name: String,
    result: String,
    log_output: String,
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

    pub fn run(&self) -> TestResult {
        match &self.mode {
            TestMode::Todo => todo!(),
            TestMode::Parse => todo!(),
            TestMode::Compile => {
                let mut compiler = mu::Compiler::new();
                let (model, diag) = compiler.compile(self.source.clone()).expect("No error");

                TestResult {
                    name: self.name.clone(),
                    result: String::from("ok"),
                    log_output: format!(
                        "{}\n-- Model --\n{}",
                        compiler.render_diag_to_string(&diag),
                        model
                    ),
                }
            }
            TestMode::Export { .. } => todo!(),
        }
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
    pub fn to_md(&self) -> mu::md::Markdown {
        let content: Vec<mu::md::Block> = todo!();
        mu::md::Markdown::page(
            self.name.clone().unwrap_or(String::from("Test result")),
            content,
        )
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
        Ok(Self {
            name: mdbook.config.title().map(|title| title.to_string()),
            test_cases: mdbook
                .code_blocks()
                .filter_map(|(path, code_block)| {
                    TestCase::from_code_block(&path, code_block).transpose()
                })
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn run(&self) -> TestResultList {
        TestResultList {
            name: self.name.clone(),
            results: self
                .test_cases
                .iter()
                .map(|test_case| test_case.run())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_case(header: &str) -> Result<Option<TestCase>, String> {
        let code_block = mu::md::CodeBlock {
            header: header.parse().expect("A code block header"),
            code: "__mu::geo2d::Circle(radius = 1mm);".into(),
            line_offset: 0,
        };
        TestCase::from_code_block(std::path::Path::new("intro/basics.md"), &code_block)
    }

    #[test]
    fn from_code_block_defaults() {
        let test_case = test_case("mu,circle").expect("Valid").expect("A test");
        assert_eq!(test_case.name, "intro::basics::circle");
        assert!(matches!(
            test_case.mode,
            TestMode::Export { path: None, .. }
        ));
        assert!(matches!(test_case.result, ExpectedTestResult::Ok));
    }

    #[test]
    fn from_code_block_parameters() {
        let test_case = test_case("mu,circle?mode=parse&result=fail")
            .expect("Valid")
            .expect("A test");
        assert!(matches!(test_case.mode, TestMode::Parse));
        assert!(matches!(test_case.result, ExpectedTestResult::Fail));

        let test_case = test_case("mu,circle?mode=export&path=circle.svg&resolution=0.5mm")
            .expect("Valid")
            .expect("A test");
        let TestMode::Export { resolution, path } = test_case.mode else {
            panic!("An export test");
        };
        assert!((resolution.linear - 0.5).abs() < 1e-6);
        assert_eq!(path, Some("circle.svg".into()));

        for (header, mode) in [("mu,a?mode=todo", "todo"), ("mu,a?mode=compile", "compile")] {
            let test_case = test_case(header).expect("Valid").expect("A test");
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
        assert!(test_case("mu").expect("Valid").is_none());
        assert!(test_case("rust,circle").expect("Valid").is_none());
    }

    #[test]
    fn from_code_block_invalid() {
        for header in [
            "mu,circle?mode=foo",
            "mu,circle?result=maybe",
            "mu,circle?resolution=fine",
        ] {
            let err = test_case(header).err().expect(header);
            assert!(err.contains("intro::basics::circle"), "{err}");
        }
    }
}
