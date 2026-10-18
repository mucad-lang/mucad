// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_driver::prelude as mu;

mod test_case;
pub use test_case::TestCase;

mod test_result;
pub use test_result::{TestResult, TestResultList};

mod test_suite;
pub use test_suite::TestSuite;

#[cfg(test)]
mod tests;

/// An artifact, which a test writes into `.test/{test_name}.{ext}` next to the markdown file.
///
/// - `mu,test?emit=model,ir,ast`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emit {
    /// The abstract syntax tree, `.ast`
    Ast,
    /// The intermediate representation, `.ir`
    Ir,
    /// The model tree, `.model`
    Model,
}

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
    /// A description for the test log.
    fn description(&self) -> &'static str {
        match self {
            TestStatus::Ok => "OK",
            TestStatus::Todo => "TODO",
            TestStatus::Fail => "FAILS",
            TestStatus::FailOk => "FAILS AS EXPECTED",
            TestStatus::OkFail => "OK BUT SHOULD FAIL",
        }
    }

    /// Returns `false` if the test failed unexpectedly or succeeded but is expected to fail.
    pub fn is_success(&self) -> bool {
        matches!(self, TestStatus::Ok | TestStatus::Todo | TestStatus::FailOk)
    }
}

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
