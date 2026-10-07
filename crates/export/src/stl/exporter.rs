// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! STL exporter.

use microcad_lang_types::{ModelTree, ModelType, Value};
use microcad_render::Renderer;

use crate::{
    ExportError, Exporter, ExporterParameters, Writer,
    stl::{
        BinaryStlWriter, WriteBinaryStl,
        ascii::{AsciiStlWriter, WriteAsciiStl},
    },
};

/// STL Exporter.
#[derive(Debug, Default)]
pub struct StlExporter {
    /// If true, the STL will be written binary format.
    pub binary: bool,
}

impl Exporter for StlExporter {
    fn export(
        &self,
        model: &ModelTree,
        parameters: &ExporterParameters,
    ) -> Result<Value, ExportError> {
        let geometry = Renderer::new().render(model)?;
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
