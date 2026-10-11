// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::Result;
use crate::prelude as mu;

use microcad_lang_base::ArtifactKind;
use microcad_lang_base::Diagnostics;
use microcad_lang_base::ToDiagnostics;
use microcad_lang_base::{Artifact, DiagRenderOptions};

use miette::IntoDiagnostic;

/// A µcad source file document.
pub struct SourceFile {
    pub source: mu::Source,
    pub ast: mu::StageResult<mu::Ast, mu::parse::ParseIssue>,
    pub ir: mu::StageResult<mu::Ir, mu::lower::LowerIssue>,
    //pub model: Option<mu::IssueResult<mu::Model>>,
}

impl SourceFile {
    /// New source file from a source.
    pub fn new(source: mu::Source) -> Self {
        Self {
            source,
            ast: mu::StageResult::default(),
            ir: mu::StageResult::default(),
        }
    }

    /// Load a source file from file.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let source = mu::Source::load(mu::locate::resolved_path(path)?).into_diagnostic()?;
        Ok(Self::new(source))
    }

    /// Emit a compiler artifact with a path. The compiler artifact extension is added automatically.
    pub fn emit(&self, path: impl AsRef<std::path::Path>, artifact_kind: &ArtifactKind) -> Result {
        match artifact_kind {
            ArtifactKind::Ast => self.ast.artifact().map(|ast| ast.emit(path)),
            ArtifactKind::Ir => self.ir.artifact().map(|ir| ir.emit(path)),
            _ => unreachable!(), //ArtifactKind::ModelTree => self.mir.artifact().map(|mir| mir.emit(path)),
                                 // ArtifactKind::SymbolTree => self.rst.artifact().map(|rst| rst.emit(path)),
        };

        Ok(())
    }

    /// Loads the code from the file specified in the `url`.
    ///
    /// # Errors
    /// Returns an error if the URL is not a valid file path or if the file cannot be read.
    pub fn load_from_file(url: mu::Url) -> mu::Result<Self> {
        // 1. Convert the URL to a local file path
        let path: std::path::PathBuf = url
            .to_file_path()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "The provided URL is not a valid local file path",
                )
            })
            .into_diagnostic()?;

        // 2. Construct and return the Source instance
        Ok(Self::new(mu::Source::load(path).into_diagnostic()?))
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> mu::Result<Self> {
        Self::load_from_file(mu::locate::to_url(
            path.as_ref().as_os_str().to_str().unwrap(),
        )?)
    }
}

impl ToDiagnostics for SourceFile {
    fn to_diagnostics(&self) -> Diagnostics {
        let mut diags = self.ast.to_diagnostics();
        diags.append(self.ir.to_diagnostics());
        diags
    }
}

impl mu::commands::GetCode for SourceFile {
    fn get_code(&self) -> Option<&str> {
        Some(self.source.code())
    }
}

impl mu::commands::SetCode for SourceFile {
    fn set_code(&mut self, code: String) -> Option<&str> {
        self.source = mu::Source::new(self.source.location.clone(), code);
        self.ast.reset();
        self.ir.reset();
        Some(self.source.code())
    }
}

impl mu::commands::Format for SourceFile {
    fn format(&mut self, params: &mu::commands::FormatParameters) -> Result<bool> {
        self.ir.reset();

        match mu::format(&self.source, params) {
            Ok(((ast, source), diags)) => {
                let changed = self.source.code() != source.code();
                self.source = source;
                self.ast = Ok((ast, diags)).into();
                Ok(changed)
            }
            Err(diags) => {
                self.ast = Err(diags).into();
                Ok(false)
            }
        }
    }
}

impl mu::commands::Sync for SourceFile {
    fn sync(&self) -> Result {
        match &self.source.path() {
            Some(path) => self.source.save(path).into_diagnostic(),
            None => Err(miette::miette!("This source does not have a file path")),
        }
    }
}

impl mu::commands::Compile for SourceFile {}

impl mu::commands::PrintDiagnostics for SourceFile {
    fn print_diagnostics(
        &self,
        _f: &mut dyn std::fmt::Write,
        _options: &DiagRenderOptions,
    ) -> std::fmt::Result {
        todo!()
    }
}
