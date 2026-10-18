// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_driver::prelude as mu;

use crate::doctest::TestStatus;

#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub source: mu::Source,
    pub log_output: String,
}
impl TestResult {
    /// Create an empty result from a todo.
    pub(crate) fn todo(name: String, source: mu::Source) -> TestResult {
        Self {
            name,
            status: TestStatus::Todo,
            source,
            log_output: String::new(),
        }
    }
}

/// A test result list with a markdown output.
#[derive(Debug)]
pub struct TestResultList {
    pub name: Option<String>,
    pub results: Vec<TestResult>,
}

impl TestResultList {
    /// Returns `true` if no test failed unexpectedly.
    pub fn is_success(&self) -> bool {
        self.results.iter().all(|result| result.status.is_success())
    }

    /// The number of tests per status, e.g. `12 ok, 2 fail_ok, 1 todo`.
    pub fn summary(&self) -> String {
        use super::TestStatus::*;
        [Ok, FailOk, Todo, Fail, OkFail]
            .into_iter()
            .filter_map(|status| {
                let count = self
                    .results
                    .iter()
                    .filter(|result| result.status == status)
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
                    .map(|result| [result.status.to_string(), result.name.clone()]),
            );

        let mut md = mu::md::Markdown::page(
            self.name.clone().unwrap_or(String::from("Test result")),
            [summary, table.into()],
        );

        let failures: Vec<_> = self
            .results
            .iter()
            .filter(|result| !result.status.is_success())
            .collect();
        if !failures.is_empty() {
            md.add_section(Section::new(2, "Failures", []));
            failures.into_iter().for_each(|result| {
                let location = Block::Paragraph(format!("Location: {}", result.source.location));
                let log = Block::CodeBlock(CodeBlock {
                    header: CodeBlockHeader {
                        lang: Some("text".into()),
                        name: None,
                        query: Default::default(),
                    },
                    code: result.log_output.trim_end().to_string(),
                    line_offset: 0,
                    path: None,
                });
                md.add_section(Section::new(3, &result.name, [location, log]));
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
