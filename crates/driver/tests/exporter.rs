// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the high-level exporter API.

use microcad_driver::{Compiler, prelude as mu};

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
        microcad_driver::resolution_from_str(s)
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
            microcad_driver::resolution_from_str(invalid).is_err(),
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
            .with_resolution(
                microcad_driver::resolution_from_str(resolution).expect("A resolution"),
            )
            .export(&compile(PART), "model.mu", Some(&path))
            .expect("An STL");
        std::fs::read_to_string(path)
            .expect("An STL file")
            .matches("facet normal")
            .count()
    };

    assert!(triangles("1mm") < triangles("0.1mm"));
}
