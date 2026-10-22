// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_driver::prelude as mu;

use crate::doctest::{TestCase, TestResult, TestResultList, TestStatus};

#[derive(Debug)]
pub struct TestSuite {
    pub name: Option<String>,
    pub test_cases: Vec<TestCase>,
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

    /// Filter test cases by wildcard.
    pub fn filter_test_cases(&mut self, wildcard: regex::Regex) {
        self.test_cases
            .retain(|test_case| wildcard.is_match(&test_case.name));
    }

    /// Insert or update the test banner of each test in the markdown files of `mdbook`.
    ///
    /// Returns the paths of the changed markdown files.
    pub fn set_banners(mdbook: &mut mu::md::MdBook) -> Vec<std::path::PathBuf> {
        mdbook
            .md_files
            .iter_mut()
            .filter_map(|(path, md)| {
                md.set_test_banners(|code_block| {
                    TestCase::from_code_block(path, code_block)
                        .ok()
                        .flatten()
                        .map(|test_case| test_case.banner())
                })
                .then(|| path.clone())
            })
            .collect()
    }

    /// Run all tests with a compiler for `config` and print the status of each test,
    /// like `cargo test`.
    pub fn run(&self, no_output: bool, config: &mu::DriverConfig) -> TestResultList {
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
                        TestResult {
                            name: test_case.name.clone(),
                            status: TestStatus::Fail,
                            source: test_case.source.clone(),
                            log_output: panic.to_string(),
                        }
                    }
                };
                eprintln!("test {} ... {}", result.name, result.status);

                if !no_output && let Err(err) = test_case.write_result_files(&result) {
                    eprintln!("Cannot write the result files of {}: {err}", result.name);
                }
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
            .filter(|result| !result.status.is_success());
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
