// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scalable Vector Graphics (SVG) export

use mucad_core::{Bounds2D, Color, Scalar};
use mucad_lang_types::{ModelTree, ModelType, Value};
use mucad_render::Renderer;

use crate::{Export, ExportError, Writer};

/// SVG Exporter.
#[derive(Debug, Default)]
pub struct SvgExporter {
    /// The render resolution.
    pub resolution: mucad_render::RenderResolution,
    /// The theme
    pub theme: Theme,
}

/// A theme for SVG export.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Background color of the drawing canvas.
    pub background: Color,
    /// Color used for grid lines.
    pub grid: Color,
    /// Color used for selected entities.
    pub selection: Color,
    /// Color used for highlighting hovered entities.
    pub highlight: Color,
    /// Default color for entities.
    pub entity: Color,
    /// Default color for entity outlines.
    pub outline: Color,
    /// Color used for active construction lines.
    pub active: Color,
    /// Color used for inactive construction lines.
    pub inactive: Color,
    /// Color for dimensions and annotations.
    pub measure: Color,
    /// Color for snapping indicators.
    pub snap_indicator: Color,
    /// Color for guidelines (e.g. inference lines).
    pub guide: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::rgb(1.0, 1.0, 1.0),
            grid: Color::rgb(0.85, 0.85, 0.85),
            selection: Color::rgb(0.0, 0.4, 0.8),
            highlight: Color::rgb(1.0, 0.6, 0.0),
            entity: Color::rgba(0.7, 0.7, 0.7, 0.7),
            outline: Color::rgb(0.1, 0.1, 0.1),
            active: Color::rgb(0.2, 0.2, 0.2),
            inactive: Color::rgb(0.8, 0.8, 0.8),
            measure: Color::rgb(0.0, 0.8, 0.8),
            snap_indicator: Color::rgb(0.0, 0.8, 0.8),
            guide: Color::rgb(0.6, 0.6, 0.6),
        }
    }
}

/// Settings for this exporter.
pub struct SvgExporterSettings {
    /// Relative padding (e.g. 0.05 = 5% = padding on each side).
    padding_factor: Scalar,
}

impl Default for SvgExporterSettings {
    fn default() -> Self {
        Self {
            padding_factor: 0.05, // 5% padding on each side.
        }
    }
}

impl SvgExporter {
    /// Generate SVG style string from theme.
    pub fn theme_to_svg_style(theme: &Theme) -> String {
        fn fill_stroke_style(
            class_name: &str,
            fill_color: Color,
            stroke_color: Color,
            stroke_width: Scalar,
        ) -> String {
            format!(
                r#"
        .{class_name} {{
            fill: {fill_color};
            stroke: {stroke_color};
            stroke-width: {stroke_width};
        }}
        "#,
                fill_color = fill_color.to_svg_color(),
                stroke_color = stroke_color.to_svg_color()
            )
        }

        fn fill_style(class_name: &str, fill: Color) -> String {
            format!(
                r#"
        .{class_name}-fill {{
            fill: {fill};
            stroke: none;
        }}
        "#,
                fill = fill.to_svg_color()
            )
        }

        fn stroke_style(class_name: &str, stroke: Color, stroke_width: Scalar) -> String {
            format!(
                r#"
        .{class_name}-stroke {{
            fill: none;
            stroke: {stroke};
            stroke-width: {stroke_width};
        }}
        "#,
                stroke = stroke.to_svg_color()
            )
        }

        let mut style = [
            ("background", theme.background, None),
            ("grid", theme.grid, Some(0.2)),
            ("measure", theme.measure, Some(0.2)),
            ("highlight", theme.highlight, Some(0.2)),
        ]
        .into_iter()
        .fold(String::new(), |mut style, item| {
            if let Some(stroke) = item.2 {
                style += &fill_stroke_style(item.0, item.1, item.1, stroke);
                style += &stroke_style(item.0, item.1, stroke)
            }
            style += &fill_style(item.0, item.1);
            style
        });

        style += &fill_stroke_style("entity", theme.entity, theme.outline, 0.4);

        style += r#"
            .active { fill-opacity: 1.0; stroke-opacity: 1.0; }
            .inactive { fill-opacity: 0.3; stroke-opacity: 0.3; }
        "#;

        style
    }
}

impl Export for SvgExporter {
    fn export(
        &self,
        model: &ModelTree,
        writer: &mut dyn std::io::Write,
    ) -> Result<Value, ExportError> {
        use crate::svg::*;
        let settings = SvgExporterSettings::default();
        let mut geometry = crate::render(
            Renderer::new().with_resolution(self.resolution.clone()),
            model,
        )?;
        let bounds = Bounds2D::from(geometry.bounds());

        let Some(content_rect) = bounds.enlarge(2.0 * settings.padding_factor).rect() else {
            return Err(ExportError::Custom("Nothing to be exported".into()));
        };
        let canvas = Canvas::new_centered(content_rect, bounds.size(), Some(1.0));
        let mut writer = SvgWriter::new(writer, canvas);
        writer.style(&SvgExporter::theme_to_svg_style(&self.theme))?;
        geometry.write_svg_mapped(&mut writer)?;
        let _ = writer.finish()?;
        Ok(Value::None)
    }

    fn model_type(&self) -> ModelType {
        ModelType::Geometry2D
    }
}
