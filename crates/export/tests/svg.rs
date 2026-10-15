// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scalable Vector Graphics (SVG) export tests

use std::str::FromStr as _;

use geo::{Translate, coord};
use microcad_builtin::{BuiltinEvalContext, mu};
use microcad_core::*;
use microcad_export::{
    Export, Writer,
    svg::{
        Background, Canvas, CenteredText, EdgeLengthMeasure, Grid, RadiusMeasure, SizeMeasure,
        SvgExporter, SvgTagAttribute, SvgWriter, Theme, WriteSvg, WriteSvgMapped,
    },
};
use microcad_lang_types::{ModelTree, Value, arguments, model::Element};
use microcad_render::Renderer;

mod common;
use common::*;

/// Renders SVG content using a draw closure, performs snapshot testing via `insta`,
/// and writes the output SVG to disk under `target/svg_outputs/<snapshot_name>.svg`.
fn assert_svg_snapshot<F>(
    snapshot_name: &str,
    canvas: impl Into<Canvas>,
    draw: F,
) -> std::io::Result<()>
where
    F: FnOnce(&mut SvgWriter<Vec<u8>>) -> std::io::Result<()>,
{
    let mut writer = SvgWriter::new(vec![], canvas.into());
    draw(&mut writer)?;

    let buffer = writer.finish()?;

    Ok(insta::with_settings!(
        {
            prepend_module_to_snapshot => false,
            snapshot_path => "../snapshots",
        },
        {
            insta::assert_binary_snapshot!(format!("{snapshot_name}.svg").as_str(), buffer);
        }
    ))
}

#[test]
fn svg_writer() -> std::io::Result<()> {
    assert_svg_snapshot(
        "svg_writer",
        Rect::new((0.0, 0.0), (100.0, 100.0)),
        |writer| {
            writer.with_attr([("style", "fill:blue")], |writer| {
                geo::Rect::new((10.0, 10.0), (20.0, 20.0)).write_svg(writer)
            })?;

            writer.with_attr([("style", "fill:red")], |writer| {
                geo2d::Circle {
                    radius: 10.0,
                    offset: (50.0, 50.0).into(),
                }
                .write_svg(writer)
            })?;

            writer.with_attr([("style", "stroke:black;")], |writer| {
                Line::new((0.0, 0.0), (100.0, 100.0)).write_svg(writer)
            })?;

            writer.with_attr([("style", "stroke:black;")], |writer| {
                Line::new((100.0, 0.0), (0.0, 100.0))
                    .shorter(6.0)
                    .write_svg(writer)
            })
        },
    )
}

/// Render a polygon with holes
#[test]
fn svg_polygon() -> std::io::Result<()> {
    assert_svg_snapshot(
        "svg_polygon",
        Rect::new((0.0, 0.0), (100.0, 100.0)),
        |writer| {
            let circle = Circle {
                radius: 10.0,
                offset: (25.0, 25.0).into(),
            }
            .to_polygon(32);

            circle.write_svg(writer)
        },
    )
}

#[test]
fn svg_canvas() -> std::io::Result<()> {
    let mut content_rect = Rect::new((0.0, 0.0), (100.0, 100.0));

    assert_svg_snapshot(
        "svg_canvas",
        Canvas::new_centered(
            content_rect.clone(),
            Size2::A4.transposed().into(),
            Some(2.0),
        ),
        |writer| {
            writer.with_attr(
                SvgTagAttribute::style(
                    Some(Color::from_str("none").expect("transparent")),
                    Some(Color::from_str("black").expect("Black color")),
                    Some(1.0),
                ),
                |writer| content_rect.write_svg_mapped(writer),
            )?;

            [
                (0.0, 0.0),
                (0.0, 100.0),
                (100.0, 0.0),
                (100.0, 100.0),
                (50.0, 50.0),
            ]
            .iter()
            .map(|p| Circle {
                radius: 2.0,
                offset: Vec2::new(p.0, p.1),
            })
            .try_for_each(|c| {
                let mut orig_c = c.clone();
                writer.with_attr(
                    SvgTagAttribute::style(
                        Some(Color::from_str("blue").expect("Color")),
                        None,
                        None,
                    ),
                    |writer| orig_c.write_svg_mapped(writer),
                )?;

                let p = Point::new(c.offset.x, c.offset.y);
                writer.with_attr(
                    SvgTagAttribute::style(
                        Some(Color::from_str("gray").expect("Color")),
                        Some(Color::from_str("none").expect("Color")),
                        None,
                    ),
                    |writer| {
                        CenteredText {
                            text: format!("({}mm,{}mm)", p.x(), p.y()),
                            rect: Rect::new(p, p),
                            font_size: 1.0,
                        }
                        .write_svg_mapped(writer)
                    },
                )
            })
        },
    )
}

#[test]
fn svg_sample_sketch() -> std::io::Result<()> {
    assert_svg_snapshot(
        "svg_sample_sketch",
        Canvas::new_centered(
            Rect::new((0.0, 0.0), (50.0, 50.0)),
            Size2::A4.transposed().into(),
            Some(2.0),
        ),
        |writer| {
            writer.style(&SvgExporter::theme_to_svg_style(&Theme::default()))?;
            let radius = 10.0;
            let width = 30.0;
            let height = 20.0;

            let circle = Circle {
                radius,
                offset: Vec2::new(width, height),
            };
            let rect = Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x: width, y: height});

            Background.write_svg(writer)?;
            Grid::default().write_svg(writer)?;

            writer.with_attr(SvgTagAttribute::class("entity-stroke inactive"), |writer| {
                let mut rect = rect.clone();
                rect.write_svg_mapped(writer)
            })?;

            writer.with_attr(SvgTagAttribute::class("entity-fill inactive"), |writer| {
                CenteredText {
                    text: "r".into(),
                    rect,
                    font_size: 4.0,
                }
                .write_svg_mapped(writer)
            })?;

            // Draw rectangle measures

            // Height measure for rect.
            writer.with_attr(SvgTagAttribute::class("measure inactive"), |writer| {
                EdgeLengthMeasure::height(&rect, 10.0, Some("height")).write_svg_mapped(writer)?;
                EdgeLengthMeasure::width(&rect, 10.0, Some("width")).write_svg_mapped(writer)
            })?;

            // Draw circle `c`.

            writer.with_attr(SvgTagAttribute::class("entity-stroke inactive"), |writer| {
                let mut circle = circle.clone();
                circle.write_svg_mapped(writer)
            })?;

            writer.with_attr(SvgTagAttribute::class("entity-fill inactive"), |writer| {
                CenteredText {
                    text: "c".into(),
                    rect: circle.calc_bounds_2d().rect().expect("Rect"),
                    font_size: 4.0,
                }
                .write_svg_mapped(writer)
            })?;

            writer.with_attr(SvgTagAttribute::class("measure inactive"), |writer| {
                RadiusMeasure::new(circle.clone(), Some("radius".into()), None)
                    .write_svg_mapped(writer)
            })?;

            // Draw intersection.
            let intersection = Geometry2D::Polygon(rect.to_polygon()).boolean_op(
                Geometry2D::Polygon(
                    Circle::circle_polygon(circle.radius, 32)
                        .translate(circle.offset.x, circle.offset.y),
                ),
                BooleanOp::Intersect,
            );

            writer.with_attr(SvgTagAttribute::class("entity-stroke active"), |writer| {
                let mut intersection = intersection.clone();
                intersection.write_svg_mapped(writer)
                // FIXME: Translation and orientation is wrong
            })?;

            writer.with_attr(SvgTagAttribute::class("measure active"), |writer| {
                let intersection = intersection.clone();
                SizeMeasure::bounds(&intersection).write_svg_mapped(writer)
            })
        },
    )
}

// Circle(42mm) - Circle(23mm);
#[test]
fn difference() -> std::io::Result<()> {
    assert_svg_snapshot(
        "svg_difference",
        Rect::new((0., 0.), (100., 100.)),
        |writer| {
            let mut ctx = BuiltinEvalContext::default();

            // {
            //     Circle(42mm);
            //     Circle(23mm);
            // }
            let mut group = ModelTree::new(Element::Group);
            group.append(circle(42.0));
            group.append(circle(23.0));
            let diff = mu::ops::subtract(arguments!(self = group), &mut ctx).expect("No error");
            let translate = mu::ops::translate(
                arguments!(
                    self = diff,
                    x = Value::mm(50.0),
                    y = Value::mm(50.0),
                    z = Value::mm(0.0)
                ),
                &mut ctx,
            )
            .expect("No error");

            let mut geometry = Renderer::new()
                .render(&translate)
                .expect("No render errors");

            geometry.write_svg_mapped(writer)
        },
    )
}

/// Export:
///
/// Rect(0mm, 0mm, 100mm, 100mm) - Rect(0mm, 0mm, 100mm, 100mm).translate(50mm, 50mm)
#[test]
fn svg_export() -> std::io::Result<()> {
    let bounds = Rect::new((0., 0.), (100., 100.));

    let mut ctx = BuiltinEvalContext::default();
    let mut group = ModelTree::new(Element::Group);

    let rect = rect(bounds);
    group.append(rect.clone());
    let translate = mu::ops::translate(
        arguments!(
            self = rect,
            x = Value::mm(50.0),
            y = Value::mm(50.0),
            z = Value::mm(0.0)
        ),
        &mut ctx,
    )
    .expect("No error");
    group.append(translate);

    let model = mu::ops::subtract(arguments!(self = group), &mut ctx).expect("No error");

    SvgExporter::default()
        .export_to_path(&model, &target_dir().join("rect_diff.svg"))
        .expect("No error");

    Ok(())
}
