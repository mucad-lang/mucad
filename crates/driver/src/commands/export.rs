// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, rc::Rc};

use crate::{
    Result,
    config::{self, ExportConfig},
};

use mucad_builtin::{Exporter, ExporterRegistry};
use mucad_lang::{
    model::{Model, OutputType},
    value::Value,
};

pub use mucad_lang::model::ExportCommand;

pub struct ExportTargets(Vec<(Model, ExportCommand)>);

impl std::fmt::Display for ExportTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|(model, export)| writeln!(f, "{model} => {}", export.filename.display()))
    }
}

#[derive(Debug)]
pub struct ExportResult {
    pub model: Model,
    pub output_path: std::path::PathBuf,
    pub value: Value,
}

impl std::fmt::Display for ExportResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val_str = match &self.value {
            Value::None => String::new(),
            v => format!(" = {v}"),
        };
        write!(f, "{}{}", self.output_path.display(), val_str)
    }
}

#[derive(Debug, derive_more::Deref)]
pub struct ExportResults(Vec<ExportResult>);

impl std::fmt::Display for ExportResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => writeln!(f, "Nothing to export"),
            1 => writeln!(f, "Exported `{}` successfully.", self.0.first().unwrap()),
            n => {
                self.0.iter().try_for_each(|r| writeln!(f, "{r}"))?;
                writeln!(f, "\n Exported {n} file(s) successfully.")
            }
        }
    }
}

impl ExportTargets {
    pub fn export(&self) -> Result<ExportResults> {
        let mut results = Vec::new();
        self.0.iter().try_for_each(|(model, export)| -> Result {
            let value = export.export(model)?;
            results.push(ExportResult {
                model: model.clone(),
                output_path: export.filename.clone(),
                value,
            });
            Ok(())
        })?;

        Ok(ExportResults(results))
    }

    pub fn list(&self) -> Result {
        for (model, attr) in &self.0 {
            eprintln!("{model} => {attr}");
        }
        Ok(())
    }
}

pub struct ExportParameters {
    /// Input file path (used as template when output path is not set).
    pub input_path: std::path::PathBuf,
    /// Output file path.
    pub output_path: Option<std::path::PathBuf>,
    /// Export configuration
    pub config: config::ExportConfig,
}

impl ExportParameters {
    pub fn default_exporter(
        &self,
        output_type: &OutputType,
        exporters: &ExporterRegistry,
    ) -> Result<Rc<dyn Exporter>> {
        use mucad_builtin::ExporterAccess;

        match output_type {
            OutputType::NotDetermined => Err(miette::miette!("Could not determine output type.")),
            OutputType::Geometry2D => Ok(exporters.exporter_by_id(&(&self.config.sketch).into())?),
            OutputType::Geometry3D => Ok(exporters.exporter_by_id(&(&self.config.part).into())?),
            OutputType::InvalidMixed => Err(miette::miette!(
                "Invalid output type, the model cannot be exported."
            )),
        }
    }

    pub fn default_export_attribute_command(
        &self,
        exporters: &ExporterRegistry,
        default_exporter: Rc<dyn Exporter>,
    ) -> Result<ExportCommand> {
        use mucad_builtin::ExporterAccess;

        Ok(match &self.output_path {
            Some(filename) => ExportCommand {
                filename: filename.clone(),
                exporter: exporters
                    .exporter_by_filename(filename)
                    .unwrap_or(default_exporter),
            },
            None => {
                let mut filename = self.input_path.clone();
                let ext = default_exporter
                    .file_extensions()
                    .first()
                    .unwrap_or(&default_exporter.id())
                    .to_string();
                filename.set_extension(&ext);

                ExportCommand {
                    filename,
                    exporter: default_exporter,
                }
            }
        })
    }

    pub fn targets(
        &self,
        model: &Model,
        default_command: ExportCommand,
    ) -> miette::Result<ExportTargets> {
        use mucad_lang::model::AttributesAccess;
        let mut models = model
            .source_file_descendants()
            .fold(Vec::new(), |mut models, model| {
                let b = model.borrow();
                models.append(
                    &mut b
                        .attributes
                        .get_exports()
                        .iter()
                        .map(|attr| (model.clone(), attr.clone()))
                        .collect(),
                );
                models
            });

        // No models with export attributes have been found.
        if models.is_empty() {
            // Add the root model with default exporters.
            models.push((model.clone(), default_command.clone()))
        }

        Ok(ExportTargets(models))
    }
}

impl From<&'static str> for ExportParameters {
    fn from(value: &'static str) -> Self {
        Self {
            input_path: PathBuf::from(value),
            output_path: Some(PathBuf::from(value)),
            config: ExportConfig::default(),
        }
    }
}

pub trait Export {
    fn get_export_targets(&self, params: impl Into<ExportParameters>) -> Result<ExportTargets>;

    fn export(&self, params: impl Into<ExportParameters>) -> Result<ExportResults> {
        self.get_export_targets(params)?.export()
    }
}
