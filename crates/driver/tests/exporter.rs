// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the high-level exporter API.

use mucad_driver::{Compiler, prelude as mu};

const SKETCH: &str = "__mu::geo2d::Circle(radius = 2mm);";

const PART: &str = "__mu::geo2d::Circle(radius = 2mm)
    .__mu::ops::extrude(height = 1mm, scale_x = 1.0, scale_y = 1.0, twist = 0°);";

fn compile(code: &str) -> mu::ModelTree {
    let mut compiler = Compiler::from_config(&mu::DriverConfig {
        no_std: true,
        ..Default::default()
    });
    let (model, diags) = compiler.compile(code).expect("A model");
    assert!(!diags.has_errors(), "{diags:?}");
    model
}

/// Without an output path, the configured exporter of the model type is used.
#[test]
fn export_default_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input = dir.path().join("model.mu");
    let exporter = mu::Exporter::new();

    let path = exporter
        .export(&compile(SKETCH), &input, None)
        .expect("An SVG");
    assert_eq!(path, dir.path().join("model.svg"));
    assert!(path.is_file());

    let path = exporter
        .export(&compile(PART), &input, None)
        .expect("An STL");
    assert_eq!(path, dir.path().join("model.stl"));
    assert!(path.is_file());
}

/// The exporter is chosen by the extension of the output path.
#[test]
fn export_output_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let output = dir.path().join("circle.svg");

    let path = mu::Exporter::new()
        .export(&compile(SKETCH), "model.mu", Some(&output))
        .expect("An SVG");
    assert_eq!(path, output);
    assert!(output.is_file());
}

/// A model cannot be exported by an exporter of another model type or an unknown exporter.
#[test]
fn export_wrong_type() {
    let dir = tempfile::tempdir().expect("temp dir");
    let exporter = mu::Exporter::new();

    let svg = dir.path().join("part.svg");
    assert!(
        exporter
            .export(&compile(PART), "model.mu", Some(&svg))
            .is_err()
    );
    assert!(!svg.exists());

    let unknown = dir.path().join("part.xyz");
    assert!(
        exporter
            .export(&compile(PART), "model.mu", Some(&unknown))
            .is_err()
    );
}

/// A resolution is a length or a percentage of the default resolution.
#[test]
fn resolution_from_str() {
    let linear = |s: &str| {
        mucad_driver::resolution_from_str(s)
            .expect("A resolution")
            .linear
    };
    // Values are parsed as fixed point numbers.
    let near = |a: f64, b: f64| (a - b).abs() < 1e-6;

    assert!(near(linear("0.05mm"), 0.05));
    assert!(near(linear("2cm"), 20.0));
    let default = mu::RenderResolution::default().linear;
    assert!(near(linear("200%"), default / 2.0));

    for invalid in ["2", "0mm", "1°", "foo"] {
        assert!(
            mucad_driver::resolution_from_str(invalid).is_err(),
            "{invalid}"
        );
    }
}

/// A finer resolution renders more triangles.
#[test]
fn export_resolution() {
    let dir = tempfile::tempdir().expect("temp dir");
    let triangles = |resolution: &str| {
        let path = dir.path().join(format!("{resolution}.stl"));
        mu::Exporter::new()
            .with_resolution(mucad_driver::resolution_from_str(resolution).expect("A resolution"))
            .export(&compile(PART), "model.mu", Some(&path))
            .expect("An STL");
        std::fs::read_to_string(path)
            .expect("An STL file")
            .matches("facet normal")
            .count()
    };

    assert!(triangles("1mm") < triangles("0.1mm"));
}

/// The bounds `(min_x, min_y, max_x, max_y)` of the path coordinates and the size of the
/// view box of an SVG.
fn svg_bounds(svg: &str) -> ((f64, f64, f64, f64), (f64, f64)) {
    let numbers = |s: &str| -> Vec<f64> {
        s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e'))
            .filter_map(|n| n.parse().ok())
            .collect()
    };
    let view_box = svg.split("viewBox='").nth(1).expect("A view box");
    let view_box = numbers(view_box.split('\'').next().expect("A view box"));

    let mut bounds = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for path in svg.split("<path d=\"").skip(1) {
        let path = path.split('"').next().expect("A path");
        // Paths of the theme, e.g. arrow markers, are not geometry.
        if !path.starts_with('M') || path.starts_with("M 0 0") {
            continue;
        }
        for point in numbers(path).chunks(2) {
            bounds.0 = bounds.0.min(point[0]);
            bounds.1 = bounds.1.min(point[1]);
            bounds.2 = bounds.2.max(point[0]);
            bounds.3 = bounds.3.max(point[1]);
        }
    }
    (bounds, (view_box[2], view_box[3]))
}

/// A sketch is centered in the SVG with a padding, also if it is not at the origin.
#[test]
fn export_svg_centered() {
    use mu::export::{Export, svg::SvgExporter};

    let model = compile(
        "__mu::geo2d::Rect(width = 20mm, height = 10mm, x = 0mm, y = 0mm)
            .__mu::ops::translate(x = 30mm, y = -40mm, z = 0mm);",
    );
    let mut svg = Vec::new();
    SvgExporter::default()
        .export(&model, &mut svg)
        .expect("An SVG");
    let svg = String::from_utf8(svg).expect("UTF-8");

    let ((min_x, min_y, max_x, max_y), (width, height)) = svg_bounds(&svg);
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    // 5% padding on each side.
    assert!(
        close(max_x - min_x, 20.0) && close(max_y - min_y, 10.0),
        "{svg}"
    );
    assert!(close(width, 22.0) && close(height, 11.0), "{svg}");
    assert!(close(min_x, width - max_x) && close(min_x, 1.0), "{svg}");
    assert!(close(min_y, height - max_y) && close(min_y, 0.5), "{svg}");
}
