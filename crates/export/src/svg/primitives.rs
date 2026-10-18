// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scalable Vector Graphics (SVG) primitives ([`WriteSvg`] trait implementations).

use std::io::Write;

use cgmath::{Deg, InnerSpace, SquareMatrix};
use geo::{CoordsIter as _, Point, Rect, Translate};
use mucad_core::*;
use mucad_render::{
    GeometryNodeData, GeometryNodeRef, GeometryOutput, GeometryOutputs, GeometryTree,
};

use crate::svg::{attributes::SvgTagAttribute, *};

impl WriteSvg for Line {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let ((x1, y1), (x2, y2)) = (self.0.x_y(), self.1.x_y());
        writer.tag(&format!(
            "line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\"",
        ))
    }
}

impl WriteSvg for Rect {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let x = self.min().x;
        let y = self.min().y;
        let width = self.width();
        let height = self.height();

        writer.tag(&format!(
            "rect x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\""
        ))
    }
}

impl WriteSvg for Bounds2D {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        if let Some(rect) = self.rect() {
            rect.write_svg(writer)
        } else {
            Ok(())
        }
    }
}

impl WriteSvg for Circle {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let r = self.radius;
        let (cx, cy) = (self.offset.x, self.offset.y);
        writer.tag(&format!("circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\""))
    }
}

impl WriteSvg for LineString {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let points = self.coords().fold(String::new(), |acc, p| {
            acc + &format!("{x},{y} ", x = p.x, y = p.y)
        });
        writer.tag(&format!("polyline points=\"{points}\""))
    }
}

impl WriteSvg for MultiLineString {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        self.iter()
            .try_for_each(|line_string| line_string.write_svg(writer))
    }
}

impl WriteSvg for Polygon {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        fn line_string_path(l: &geo2d::LineString) -> String {
            l.points()
                .enumerate()
                .fold(String::new(), |acc, (i, point)| {
                    let (x, y) = point.x_y();
                    let mut s = String::new();
                    s += if i == 0 { "M" } else { "L" };
                    s += &format!("{x},{y}");
                    if i == l.coords_count() - 1 {
                        s += " Z ";
                    }
                    acc + &s
                })
        }

        let exterior = line_string_path(self.exterior());
        let interior = self
            .interiors()
            .iter()
            .map(line_string_path)
            .fold(String::new(), |acc, s| acc + &s);

        writer.tag(&format!("path d=\"{exterior} {interior}\""))
    }
}

impl WriteSvg for MultiPolygon {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        self.iter()
            .try_for_each(|polygon| polygon.write_svg(writer))
    }
}

impl WriteSvg for Geometries2D {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        self.iter().try_for_each(|geo| geo.write_svg(writer))
    }
}

impl WriteSvg for Geometry2D {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        match self {
            Geometry2D::LineString(line_string) => line_string.write_svg(writer),
            Geometry2D::MultiLineString(multi_line_string) => multi_line_string.write_svg(writer),
            Geometry2D::Polygon(polygon) => polygon.write_svg(writer),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.write_svg(writer),
            Geometry2D::Line(edge) => edge.write_svg(writer),
            Geometry2D::Collection(collection) => collection.write_svg(writer),
        }
    }
}

impl WriteSvg for Geometry {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        match &self {
            Geometry::Geometry2D(geo2d) => geo2d.write_svg(writer),
            Geometry::Geometry3D(_) => todo!("Error handling"),
        }
    }
}

impl WriteSvg for GeometryOutput {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        writer.with_attr(
            SvgTagAttributes::from(self.attr.clone()).insert(SvgTagAttribute::class("entity")),
            |writer| self.geometry.write_svg(writer),
        )
    }
}

impl WriteSvg for GeometryOutputs {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        self.iter().try_for_each(|output| output.write_svg(writer))
    }
}

impl WriteSvg for GeometryNodeData {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let mut attr = SvgTagAttributes::new().insert(SvgTagAttribute::class("entity"));

        if !self.local_matrix.is_identity() {
            attr = attr.insert(SvgTagAttribute::Transform(mat4_to_mat3(&self.local_matrix)));
        }

        writer.begin_group(attr)?;
        self.outputs.write_svg(writer)?;
        writer.end_group()
    }
}

impl WriteSvg for GeometryTree {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        fn recurse<W: Write>(
            node: GeometryNodeRef,
            writer: &mut SvgWriter<W>,
        ) -> std::io::Result<()> {
            if node.outputs.is_empty() {
                node.children().try_for_each(|child| recurse(child, writer))
            } else {
                node.outputs.write_svg(writer)
            }
        }

        recurse(self.root(), writer)
    }
}

/// A struct for drawing a centered text.
pub struct CenteredText {
    /// The actual text.
    pub text: String,
    /// Bounding rectangle
    pub rect: Rect,
    /// Font size in mm.
    pub font_size: Scalar,
}

impl WriteSvg for CenteredText {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let (x, y) = self.rect.center().x_y();
        let font_size = self.font_size;
        writer.open_tag(
            format!(r#"text x="{x}" y="{y}" font-size="{font_size}mm" dominant-baseline="middle" text-anchor="middle""#,)
                .as_str(),
        )?;
        writer.with_indent(&self.text)?;
        writer.close_tag("text")
    }
}

/// A struct for drawing a grid.
pub struct Grid {
    /// Grid bounds.
    pub bounds: Bounds2D,

    /// Grid cell size.
    pub cell_size: Size2,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            bounds: Bounds2D::default(),
            cell_size: Size2 {
                width: 10.0,
                height: 10.0,
            },
        }
    }
}

impl WriteSvg for Grid {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let rect = self.bounds.rect().unwrap_or(writer.canvas().rect);
        writer.begin_group(SvgTagAttribute::class("grid-stroke"))?;

        rect.write_svg(writer)?;

        let mut left = rect.min().x;
        let right = rect.max().x;
        while left <= right {
            Line(
                geo::Point::new(left, rect.min().y),
                geo::Point::new(left, rect.max().y),
            )
            .write_svg(writer)?;
            let mut w = self.cell_size.width;
            w.map_to_canvas(writer.canvas());
            left += w;
        }

        let mut bottom = rect.min().y;
        let top = rect.max().y;
        while bottom <= top {
            Line(
                geo::Point::new(rect.min().x, bottom),
                geo::Point::new(rect.max().x, bottom),
            )
            .write_svg(writer)?;
            let mut h = self.cell_size.height;
            h.map_to_canvas(writer.canvas());
            bottom += h;
        }

        writer.end_group()?;

        Ok(())
    }
}

/// A struct for drawing a background.
pub struct Background;

impl WriteSvg for Background {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let x = 0;
        let y = 0;
        let width = writer.canvas().size.width;
        let height = writer.canvas().size.height;

        writer.tag(
            &format!("rect class=\"background-fill\" x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\""),
        )
    }
}

/// A measure to measure a length of an edge.
pub struct EdgeLengthMeasure {
    // Optional name for this measure.
    name: Option<String>,
    // Original Length
    length: Scalar,
    // Edge.
    edge: Line,
    // Offset (default = 10mm).
    offset: Scalar,
}

impl EdgeLengthMeasure {
    /// Height measure of a rect.
    pub fn height(rect: &Rect, offset: Scalar, name: Option<&str>) -> Self {
        let edge = Line(
            geo::Point::new(rect.min().x, rect.min().y),
            geo::Point::new(rect.min().x, rect.max().y),
        );
        Self {
            name: name.map(|s| s.into()),
            length: edge.vec().magnitude(),
            edge,
            offset: -offset,
        }
    }

    /// Width measure of a rect.
    pub fn width(rect: &Rect, offset: Scalar, name: Option<&str>) -> Self {
        let edge = Line(
            geo::Point::new(rect.min().x, rect.min().y),
            geo::Point::new(rect.max().x, rect.min().y),
        );

        Self {
            name: name.map(|s| s.into()),
            length: edge.vec().magnitude(),
            edge,
            offset,
        }
    }
}

impl MapToCanvas for EdgeLengthMeasure {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.edge.map_to_canvas(canvas);
        self.offset.map_to_canvas(canvas);
    }
}

impl WriteSvg for EdgeLengthMeasure {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        let edge_length = self.edge.vec().magnitude();

        use attributes::SvgTagAttribute::*;

        writer.begin_group(SvgTagAttribute::Transform(self.edge.matrix()))?;

        let center = self.offset / 2.0;
        let bottom_left = Point::new(0.0, 0.0);
        let bottom_right = Point::new(edge_length, 0.0);
        let top_left = Point::new(0.0, center);
        let top_right = Point::new(edge_length, center);

        writer.begin_group(SvgTagAttribute::class("measure"))?;
        Line(bottom_left, Point::new(0.0, center * 1.5)).write_svg(writer)?;
        Line(bottom_right, Point::new(edge_length, center * 1.5)).write_svg(writer)?;

        writer.with_attr(
            SvgTagAttributes::new()
                .insert(MarkerStart("arrow".into()))
                .insert(MarkerEnd("arrow".into())),
            |writer| Line(top_left, top_right).shorter(1.5).write_svg(writer),
        )?;
        writer.end_group()?;

        writer.with_attr(SvgTagAttribute::class("measure-fill"), |writer| {
            CenteredText {
                text: format!(
                    "{name}{length:.2}mm",
                    name = match &self.name {
                        Some(name) => format!("{name} = "),
                        None => String::new(),
                    },
                    length = self.length
                ),
                rect: Rect::new(bottom_left, top_right).translate(0.0, center),
                font_size: 2.0,
            }
            .write_svg(writer)
        })?;

        writer.end_group()
    }
}

/// A radius measure with an offset.
pub struct RadiusMeasure {
    /// Circle to measure.
    pub circle: Circle,
    /// Original radius to measure.
    pub radius: Scalar,
    /// Name of this measurement.
    pub name: Option<String>,
    /// Angle of the measurement.
    pub angle: Deg<Scalar>,
}

impl RadiusMeasure {
    /// Create new radius measure.
    pub fn new(circle: Circle, name: Option<String>, angle: Option<Deg<Scalar>>) -> Self {
        Self {
            radius: circle.radius,
            circle,
            name,
            angle: angle.unwrap_or(Deg(-45.0)),
        }
    }
}

impl MapToCanvas for RadiusMeasure {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.circle.map_to_canvas(canvas);
    }
}

impl WriteSvg for RadiusMeasure {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        writer.begin_group(SvgTagAttributes::new())?;

        let edge = Line::radius_edge(&self.circle, &self.angle.into());
        writer.with_attr(
            SvgTagAttributes::new()
                .insert(SvgTagAttribute::MarkerEnd("arrow".into()))
                .insert(SvgTagAttribute::class("measure")),
            |writer| edge.shorter(1.5).write_svg(writer),
        )?;

        let center = edge.center();
        writer.with_attr(SvgTagAttribute::class("measure-fill"), |writer| {
            CenteredText {
                text: format!(
                    "{name}{radius:.2}mm",
                    name = match &self.name {
                        Some(name) => format!("{name} = "),
                        None => String::new(),
                    },
                    radius = self.radius,
                ),
                rect: Rect::new(center, center),
                font_size: 2.0,
            }
            .write_svg(writer)
        })?;

        writer.end_group()
    }
}

/// Size measure of a bounds.
pub struct SizeMeasure {
    bounds: Bounds2D,
    /// Width measure
    width: Option<EdgeLengthMeasure>,
    /// Height measure
    height: Option<EdgeLengthMeasure>,
}

impl SizeMeasure {
    /// Size measure for something that has bounds.
    pub fn bounds<T: CalcBounds2D>(bounds: &T) -> Self {
        let bounds = bounds.calc_bounds_2d();

        if let Some(rect) = bounds.rect() {
            Self {
                bounds: bounds.clone(),
                width: Some(EdgeLengthMeasure::width(&rect, 7.0, None)),
                height: Some(EdgeLengthMeasure::height(&rect, 7.0, None)),
            }
        } else {
            Self {
                bounds: bounds.clone(),
                width: None,
                height: None,
            }
        }
    }
}

impl MapToCanvas for SizeMeasure {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.bounds.map_to_canvas(canvas);
        if let Some(width) = self.width.as_mut() {
            width.map_to_canvas(canvas)
        }
        if let Some(height) = self.height.as_mut() {
            height.map_to_canvas(canvas)
        }
    }
}

impl WriteSvg for SizeMeasure {
    fn write_svg<W: Write>(&self, writer: &mut SvgWriter<W>) -> std::io::Result<()> {
        if let Some(width) = &self.width {
            width.write_svg(writer)?;
        }
        if let Some(height) = &self.height {
            height.write_svg(writer)?;
        }
        Ok(())
    }
}
