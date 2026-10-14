// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad model exporter

use miette::IntoDiagnostic;

use crate::prelude as mu;
use mu::export::{Export, stl::StlExporter, svg::SvgExporter};
use mu::types::{ModelType, model::NodeExt};

/// Exports a compiled model into a file, e.g. an SVG for a sketch or an STL for a part.
#[derive(Debug, Default)]
pub struct Exporter {
    config: mu::ExportConfig,
    resolution: mu::RenderResolution,
}

impl Exporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: mu::ExportConfig) -> Self {
        self.config = config;
        self
    }

    /// Render with a resolution other than the default.
    pub fn with_resolution(mut self, resolution: mu::RenderResolution) -> Self {
        self.resolution = resolution;
        self
    }
}

impl Exporter {
    /// Export a model compiled from `input` and return the path of the exported file.
    ///
    /// The exporter is chosen by the extension of `output`. Without `output`, the file is
    /// written next to `input` with the extension of the configured exporter for the model type.
    pub fn export(
        &self,
        model: &mu::ModelTree,
        input: impl AsRef<std::path::Path>,
        output: Option<&std::path::Path>,
    ) -> mu::Result<std::path::PathBuf> {
        let path = self.output_path(model, input, output)?;
        let exporter = self.exporter(&path)?;

        let model_type = model.root().deduce_output_type();
        if model_type != exporter.model_type() {
            return Err(miette::miette!(
                "Cannot export a {model_type} model into {path:?}"
            ));
        }

        exporter.export_to_path(model, &path).into_diagnostic()?;
        Ok(path)
    }

    /// The path of the file a model compiled from `input` is exported into.
    pub fn output_path(
        &self,
        model: &mu::ModelTree,
        input: impl AsRef<std::path::Path>,
        output: Option<&std::path::Path>,
    ) -> mu::Result<std::path::PathBuf> {
        if let Some(output) = output {
            return Ok(output.to_path_buf());
        }

        let extension = match model.root().deduce_output_type() {
            ModelType::Geometry2D => &self.config.sketch,
            ModelType::Geometry3D => &self.config.part,
            model_type => {
                return Err(miette::miette!("Cannot export a {model_type} model"));
            }
        };
        Ok(input.as_ref().with_extension(extension))
    }
}

impl Exporter {
    /// The exporter for the extension of `path`.
    fn exporter(&self, path: &std::path::Path) -> mu::Result<Box<dyn Export>> {
        let resolution = self.resolution.clone();
        let extension = path.extension().and_then(|ext| ext.to_str());
        match extension {
            Some("svg") => Ok(Box::new(SvgExporter {
                resolution,
                ..Default::default()
            })),
            Some("stl") => Ok(Box::new(StlExporter {
                resolution,
                ..Default::default()
            })),
            _ => Err(miette::miette!("No exporter for {path:?}")),
        }
    }
}
