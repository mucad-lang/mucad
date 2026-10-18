// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_driver::prelude::{self as mu, Artifact, traits::RenderDiag};

use crate::doctest::{Emit, ExpectedTestResult, Panic, TestMode, TestResult, TestStatus};

/// Directory, into which tests export their models.
const OUTPUT_DIR: &str = "target/doctest";

/// Diagnostics are rendered without colors, because they appear in markdown.
const LOG_OPTIONS: mu::base::DiagRenderOptions = mu::base::DiagRenderOptions {
    color: false,
    unicode: false,
};

#[derive(Debug)]
pub struct TestCase {
    pub(crate) name: String,
    pub mode: TestMode,
    /// Expected test result
    pub result: ExpectedTestResult,
    /// Artifacts to write into the test directory.
    pub emit: Vec<Emit>,
    pub source: mu::Source,
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

        let emit = match param("emit") {
            Some(emit) => emit
                .split(',')
                .map(|artifact| match artifact {
                    "ast" => Ok(Emit::Ast),
                    "ir" => Ok(Emit::Ir),
                    "model" => Ok(Emit::Model),
                    _ => Err(invalid("emit", &emit)),
                })
                .collect::<Result<_, _>>()?,
            None => Vec::new(),
        };

        Ok(Some(Self {
            name,
            mode,
            result,
            emit,
            source: code_block.source(),
        }))
    }

    /// Run the test with `compiler` and compare the outcome with the expected result.
    ///
    /// A panic, e.g. a `todo!()` in the compiler, is returned as error.
    pub fn run(&self, compiler: &mut mu::Compiler) -> Result<TestResult, Panic> {
        if let TestMode::Todo = self.mode {
            return Ok(TestResult::todo(self.name.clone(), self.source.clone()));
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
        Ok(TestResult {
            name: self.name.clone(),
            status,
            source: self.source.clone(),
            log_output,
        })
    }

    /// Execute the test and return its log: `Ok` if it succeeded, `Err` if it failed.
    fn execute(&self, compiler: &mut mu::Compiler) -> Result<String, String> {
        match &self.mode {
            TestMode::Todo => Ok(String::new()),
            TestMode::Parse => self.parse(),
            TestMode::Compile => self.compile(compiler).map(|(_, log)| log),
            TestMode::Export { resolution, path } => {
                let (model, log) = self.compile(compiler)?;

                // Code without geometry (e.g. only values or an empty sketch) is only compiled.
                if !model.has_primitives() {
                    return Ok(format!("{log}\nNo model to export."));
                }

                match self.export(&model, resolution, path.as_deref()) {
                    Ok(path) => Ok(format!("{log}\nExported '{}'.", path.display())),
                    // The message only, the log is written without colors.
                    Err(err) => Err(format!("{log}\n{err}")),
                }
            }
        }
    }

    /// The name of the code block, with which the test name ends.
    fn block_name(&self) -> &str {
        self.name.rsplit("::").next().unwrap_or(&self.name)
    }

    /// The markdown of the test banner, which links to the test result.
    pub fn banner(&self) -> String {
        mu::md::CodeBlockHeader::test_banner_string(self.block_name())
    }

    /// The directory `.test` next to the markdown file, which contains the test results.
    ///
    /// It is `None` if the source has no path, e.g. if it is not loaded from a markdown file.
    fn test_dir(&self) -> Option<std::path::PathBuf> {
        let md_file = self.source.path()?;
        Some(
            md_file
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .join(".test"),
        )
    }

    /// Write the files the banner links to into `.test` next to the markdown file:
    /// an image for the test status and the test log.
    ///
    /// Does nothing if the source has no path, e.g. if it is not loaded from a markdown file.
    pub fn write_result_files(&self, result: &TestResult) -> std::io::Result<()> {
        let Some(dir) = self.test_dir() else {
            return Ok(());
        };
        std::fs::create_dir_all(&dir)?;

        // Link the image for the status, e.g. `xtask/images/ok.svg`.
        let image = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("images")
            .join(format!("{}.svg", result.status));
        let banner = dir.join(format!("{}.svg", self.block_name()));
        let _ = std::fs::remove_file(&banner);
        std::fs::hard_link(&image, &banner)
            .or_else(|_| std::fs::copy(&image, &banner).map(|_| ()))?;

        std::fs::write(
            dir.join(format!("{}.log", self.block_name())),
            self.log(result),
        )
    }

    /// The test log with the test parameters, the code, the output and the test result.
    fn log(&self, result: &TestResult) -> String {
        let line_offset = self.source.location.line_offset.unwrap_or_default() as usize;
        let code = self
            .source
            .code()
            .lines()
            .enumerate()
            .map(|(i, line)| format!("  {:>3}:   {line}\n", line_offset + i + 1))
            .collect::<String>();
        // The first line of the code, e.g. `intro.md:33`.
        let source_file = match self.source.path() {
            Some(path) => format!("{}:{}", path.display(), line_offset + 1),
            None => self.source.location.to_string(),
        };
        let expected = match self.result {
            ExpectedTestResult::Ok => "ok",
            ExpectedTestResult::Fail => "fail",
        };

        format!(
            "-- Test --\n\
            {:>17}: {}\n\
            {:>17}: {expected}\n\
            {:>17}: {}\n\n\
            -- Code --\n\n{code}\n\
            -- Output --\n{}\n\
            -- Test Result --\n{}\n",
            "Test name",
            self.name,
            "Expected result",
            "Source file",
            source_file,
            result.log_output.trim_end(),
            result.status.description(),
        )
    }

    /// Parse the source and return the diagnostics and the emitted AST.
    fn parse(&self) -> Result<String, String> {
        let (ast, diags) = match mu::parse(&self.source) {
            Ok((ast, issues)) => (Some(ast), issues.into_diagnostics()),
            Err(issues) => (None, issues.into_diagnostics()),
        };
        let log = diags
            .render_to_string(&&self.source, &LOG_OPTIONS)
            .unwrap_or_default();
        let log = log
            + &self.emit_log(|emit, path| match (emit, &ast) {
                (Emit::Ast, Some(ast)) => ast.emit(path),
                (Emit::Ast, None) => Err(mu::report("No AST to emit")),
                (Emit::Ir | Emit::Model, _) => {
                    Err(mu::report("Only the AST is emitted when parsing"))
                }
            });
        match diags.has_errors() {
            true => Err(log),
            false => Ok(log),
        }
    }

    /// Compile the source and return the model, the diagnostics and the emitted artifacts.
    ///
    /// Like in the CLI, a model with errors is a failure.
    fn compile(&self, compiler: &mut mu::Compiler) -> Result<(mu::ModelTree, String), String> {
        let (model, diags) = match compiler.compile(self.source.clone()) {
            Ok((model, diags)) => (Some(model), diags),
            Err(diags) => (None, diags),
        };
        let id = self.source.hash_id();
        let log = compiler.render_diag_to_string_with(&diags, &LOG_OPTIONS)
            + &self.emit_log(|emit, path| match (emit, &model) {
                (Emit::Ast, _) => compiler.emit_ast(id, path),
                (Emit::Ir, _) => compiler.emit_ir(id, path),
                (Emit::Model, Some(model)) => model.emit(path),
                (Emit::Model, None) => Err(mu::report("No model to emit")),
            });
        match model {
            Some(model) if !diags.has_errors() => Ok((model, log)),
            _ => Err(log),
        }
    }

    /// Write the artifacts of [`TestCase::emit`] with `emit` into the test directory,
    /// e.g. `.test/circle.ast`, and return their file names for the log.
    fn emit_log(
        &self,
        emit: impl Fn(Emit, &std::path::Path) -> mu::Result<std::path::PathBuf>,
    ) -> String {
        let Some(dir) = self.test_dir().filter(|_| !self.emit.is_empty()) else {
            return String::new();
        };
        if let Err(err) = std::fs::create_dir_all(&dir) {
            return format!("\nCannot create '{}': {err}", dir.display());
        }
        let path = dir.join(self.block_name());
        self.emit
            .iter()
            .map(|artifact| match emit(*artifact, &path) {
                Ok(path) => format!(
                    "\nEmitted '{}'.",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                Err(err) => format!("\n{err}"),
            })
            .collect()
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
