// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad model exporter

use crate::prelude as mu;
use mu::export::{Export, stl::StlExporter, svg::SvgExporter};
use mu::types::{ModelType, model::NodeExt};

/// A file to export a model into, e.g. of an export attribute: `#[export = "logo.svg"]`.
pub struct ExportFile<'tree> {
    /// The path of the file, its extension selects the exporter.
    pub path: std::path::PathBuf,
    /// The render resolution.
    pub resolution: mu::RenderResolution,
    /// The model to export.
    pub tree: mu::types::ModelNodeRef<'tree>,
}

/// Exports a compiled model into a file, e.g. an SVG for a sketch or an STL for a part.
#[derive(Debug, Default)]
pub struct Exporter {
    config: mu::ExporterConfig,
    resolution: mu::RenderResolution,
}

impl Exporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: mu::ExporterConfig) -> Self {
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
        self.write(model, &path, &self.resolution)?;
        Ok(path)
    }

    /// The files of the export attributes of the source statements in a model compiled from
    /// `input`, e.g. `#[export = "logo.svg"]`.
    ///
    /// Named statements (`logo = …`) are the properties of the root model, unnamed statements
    /// its children. Models inside them are not searched, e.g. a property used by a child.
    /// Without file, a named statement is exported into `<input>__<name>` with the extension of
    /// the configured exporter for its model type, e.g. `logo__shape.svg`.
    /// Relative files are placed next to `input`.
    pub fn export_files<'tree>(
        &self,
        model: &'tree mu::ModelTree,
        input: impl AsRef<std::path::Path>,
    ) -> mu::Result<Vec<ExportFile<'tree>>> {
        use mu::types::model::Attribute;

        let input = input.as_ref();
        let dir = input.parent().unwrap_or(std::path::Path::new(""));
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();

        let root = model.root();
        // The model lives as long as the tree, not as the node reference `root`.
        let named =
            root.get()
                .properties
                .iter()
                .filter_map(|(name, property)| match &property.value {
                    mu::Value::Model(tree) => Some((Some(name), tree.root())),
                    _ => None,
                });
        let unnamed = root.children().map(|node| (None, node));

        let mut files = Vec::new();
        for (name, node) in named.chain(unnamed) {
            let exports = node.attr.iter().filter_map(|attr| match attr {
                Attribute::Export(export) => Some(export),
                _ => None,
            });
            for export in exports {
                let path = match (&export.path, name) {
                    (Some(path), _) => dir.join(path),
                    (None, Some(name)) => {
                        let extension = self.extension(node.deduce_output_type())?;
                        dir.join(format!("{stem}__{name}.{extension}"))
                    }
                    // Lowering reports a missing file of an unnamed statement.
                    (None, None) => {
                        return Err(miette::miette!(
                            "An export of a statement without name requires a file name"
                        ));
                    }
                };
                files.push(ExportFile {
                    path,
                    resolution: self.resolution.clone(),
                    tree: node,
                });
            }
        }
        Ok(files)
    }

    /// Export the model of an export file and return its path.
    pub fn export_file(&self, file: &ExportFile) -> mu::Result<std::path::PathBuf> {
        self.write(
            &mu::ModelTree::from(file.tree),
            &file.path,
            &file.resolution,
        )?;
        Ok(file.path.clone())
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

        let extension = self.extension(model.root().deduce_output_type())?;
        Ok(input.as_ref().with_extension(extension))
    }
}

impl Exporter {
    /// The extension of the configured exporter for a model type, e.g. `svg` for a sketch.
    fn extension(&self, model_type: ModelType) -> mu::Result<&str> {
        match model_type {
            ModelType::Geometry2D => Ok(&self.config.sketch),
            ModelType::Geometry3D => Ok(&self.config.part),
            model_type => Err(miette::miette!("Cannot export a {model_type} model")),
        }
    }

    /// Export `model` into `path` with the exporter for its extension.
    fn write(
        &self,
        model: &mu::ModelTree,
        path: &std::path::Path,
        resolution: &mu::RenderResolution,
    ) -> mu::Result<()> {
        let exporter = self.exporter(path, resolution)?;

        let model_type = model.root().deduce_output_type();
        if model_type != exporter.model_type() {
            return Err(miette::miette!(
                "Cannot export a {model_type} model into {path:?}"
            ));
        }

        // A report, which keeps related diagnostics, e.g. render errors.
        exporter
            .export_to_path(model, path)
            .map_err(miette::Report::new)?;
        Ok(())
    }

    /// The exporter for the extension of `path`.
    fn exporter(
        &self,
        path: &std::path::Path,
        resolution: &mu::RenderResolution,
    ) -> mu::Result<Box<dyn Export>> {
        let resolution = resolution.clone();
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
