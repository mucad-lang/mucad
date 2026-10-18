// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export models to files

use miette::Diagnostic;
use mucad_lang_types::{ModelTree, ModelType, Value};
use mucad_render::{RenderError, RenderIssue, Renderer};
use thiserror::Error;

pub mod stl;
pub mod svg;
pub mod wkt;

/// A lifecycle-aware writer trait for exporters.
pub trait Writer {
    /// The underlying stream/output type returned by `finish()`.
    type Output;

    /// Optional initialization/header writer (e.g., XML headers, doc preambles).
    /// Called automatically before writing body content if not called explicitly.
    fn begin(&mut self) -> std::io::Result<()>;

    /// Writes remaining structural elements (e.g., closing tags, footers) and flushes,
    /// without consuming `self`. Must be idempotent.
    fn finalize(&mut self) -> std::io::Result<()>;

    /// Extract the inner stream/writer.
    fn into_inner(self) -> std::io::Result<Self::Output>;

    /// Default blanket implementation of finish for any Writer.
    fn finish(mut self) -> std::io::Result<Self::Output>
    where
        Self: Sized,
    {
        self.finalize()?;
        self.into_inner()
    }
}

/// An export error
#[derive(Debug, Error, Diagnostic)]
pub enum ExportError {
    /// Custom error.
    #[error("{0}")]
    Custom(String),

    /// I/O Error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Format error
    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),

    /// Errors when rendering the model, e.g. a built-in without render hook.
    ///
    /// The render errors are shown as related diagnostics.
    #[error("The model cannot be rendered without errors")]
    RenderErrors(#[related] Vec<RenderError>),
}

/// Render the model for an export.
///
/// Render errors make the export fail, even if rendering continued.
fn render(
    mut renderer: Renderer,
    model: &ModelTree,
) -> Result<mucad_render::GeometryTree, ExportError> {
    match renderer.render(model) {
        Ok((geometry, issues)) if !issues.has_errors() => Ok(geometry),
        Ok((_, issues)) | Err(issues) => Err(ExportError::RenderErrors(
            issues
                .into_iter()
                .filter_map(|issue| match issue {
                    RenderIssue::Error(err) => Some(err),
                    RenderIssue::Warning(_) | RenderIssue::Advice(_) => None,
                })
                .collect(),
        )),
    }
}

/// An `Exporter` exports a `ModelTree`.
pub trait Export {
    /// The export function.
    fn export(
        &self,
        model: &ModelTree,
        writer: &mut dyn std::io::Write,
    ) -> Result<Value, ExportError>;

    /// Export the model tree to a path.
    fn export_to_path(
        &self,
        model: &ModelTree,
        path: &std::path::Path,
    ) -> Result<Value, ExportError> {
        let mut buffer = Vec::new();
        let value = self.export(model, &mut buffer)?;
        std::fs::write(path, buffer)?;
        Ok(value)
    }

    /// The model type this export is supposed to use.
    fn model_type(&self) -> ModelType {
        ModelType::default()
    }
}
