// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! STL exporter.

use mucad_lang_types::{ModelTree, ModelType, Value};
use mucad_render::Renderer;

use crate::{
    Export, ExportError, ExporterParameters, Writer,
    stl::{
        BinaryStlWriter, WriteBinaryStl,
        ascii::{AsciiStlWriter, WriteAsciiStl},
    },
};

/// STL Exporter.
#[derive(Debug, Default)]
pub struct StlExporter {
    /// The render resolution.
    pub resolution: mucad_render::RenderResolution,
    /// If true, the STL will be written binary format.
    pub binary: bool,
}

impl Export for StlExporter {
    fn export(
        &self,
        model: &ModelTree,
        parameters: &ExporterParameters,
    ) -> Result<Value, ExportError> {
        let geometry = crate::render(
            Renderer::new().with_resolution(self.resolution.clone()),
            model,
        )?;
        let f = std::fs::File::create(&parameters.path)?;

        if self.binary {
            let mut writer = BinaryStlWriter::new(f);
            geometry.write_binary_stl(&mut writer)?;
            writer.finalize()?;
        } else {
            let mut writer = AsciiStlWriter::new(f);
            geometry.write_ascii_stl(&mut writer)?;
            writer.finalize()?;
        }

        Ok(Value::None)
    }

    fn model_type(&self) -> ModelType {
        ModelType::Geometry3D
    }
}
