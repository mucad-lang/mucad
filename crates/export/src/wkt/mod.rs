// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export 2D models to Well-Known Text (WKT).

use std::io::Write;

use geo::line_string;
use microcad_core::{Geometries2D, Geometry, Geometry2D};
use microcad_lang_types::{ModelTree, ModelType, Value};

use microcad_render::{
    GeometryNodeData, GeometryNodeRef, GeometryOutput, GeometryOutputs, GeometryTree, Renderer,
};
use wkt::ToWkt;

use crate::{Export, ExportError, ExporterParameters};

/// WKT Exporter.
pub struct WktExporter;

trait WriteWkt {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()>;
}

impl WriteWkt for Geometries2D {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writeln!(writer, "GEOMETRYCOLLECTION(")?;
        self.iter().try_for_each(|geo| geo.write_wkt(writer))?;
        writeln!(writer, ")")
    }
}

impl WriteWkt for Geometry2D {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self {
            Geometry2D::LineString(line_string) => {
                writeln!(writer, "{}", line_string.wkt_string())
            }
            Geometry2D::MultiLineString(multi_line_string) => {
                writeln!(writer, "{}", multi_line_string.wkt_string())
            }
            Geometry2D::Polygon(polygon) => {
                writeln!(writer, "{}", polygon.wkt_string())
            }
            Geometry2D::MultiPolygon(multi_polygon) => {
                writeln!(writer, "{}", multi_polygon.wkt_string())
            }
            Geometry2D::Line(line) => {
                writeln!(
                    writer,
                    "{}",
                    line_string![line.0.into(), line.1.into()].wkt_string()
                )
            }
            Geometry2D::Collection(collection) => collection.write_wkt(writer),
        }
    }
}

impl WriteWkt for Geometry {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Geometry::Geometry2D(geo2d) => geo2d.write_wkt(writer),
            Geometry::Geometry3D(_) => todo!(),
        }
    }
}

impl WriteWkt for GeometryOutput {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.geometry.write_wkt(writer)
    }
}

impl WriteWkt for GeometryOutputs {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.iter().try_for_each(|geo| geo.write_wkt(writer))
    }
}

impl WriteWkt for GeometryNodeData {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.outputs.write_wkt(writer)
    }
}

impl WriteWkt for GeometryTree {
    fn write_wkt(&self, writer: &mut impl Write) -> std::io::Result<()> {
        fn recurse(node: GeometryNodeRef, writer: &mut impl Write) -> std::io::Result<()> {
            if node.outputs.is_empty() {
                node.children().try_for_each(|child| recurse(child, writer))
            } else {
                node.outputs.write_wkt(writer)
            }
        }

        recurse(self.root(), writer)
    }
}

impl Export for WktExporter {
    fn export(
        &self,
        model: &ModelTree,
        parameters: &ExporterParameters,
    ) -> Result<Value, ExportError> {
        let geometry = crate::render(Renderer::new(), model)?;
        let mut f = std::fs::File::create(&parameters.path)?;
        geometry.write_wkt(&mut f)?;

        Ok(Value::None)
    }

    fn model_type(&self) -> ModelType {
        ModelType::Geometry2D
    }
}
