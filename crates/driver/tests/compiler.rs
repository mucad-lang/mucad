// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the high-level compiler API.

use microcad_driver::{Compiler, prelude as mu};
use mu::base::DisplayWithCtx;

/// A sketch and an operation, called as method on the sketch.
const DISC: &str = r#"
sketch Disc(radius: Length) {
    __mu::geo2d::Circle(radius);
}

op Raise(height: Length) {
    @input.__mu::ops::extrude(height, scale_x = 1.0, scale_y = 1.0, twist = 0°);
}

Disc(radius = 4mm).Raise(height = 2mm);
"#;

fn display(model: &mu::ModelTree) -> String {
    model.to_string_with_ctx(&mu::builtin::BuiltinLibrary::new())
}

/// A compiler without standard library, independent of any installation.
fn compiler() -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        no_std: true,
        ..Default::default()
    })
}

/// A compiler with the standard library of this repository (relative to `crates/driver`).
fn std_compiler() -> Compiler {
    Compiler::from_config(&mu::DriverConfig {
        search_paths: vec!["../std/lib".into()],
        ..Default::default()
    })
}

#[test]
fn compile_source() {
    let mut compiler = compiler();
    let (model, diags) = compiler.compile(mu::Source::from(DISC)).expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_source", display(&model));
}

#[test]
fn compile_unresolved_symbol() {
    let mut compiler = compiler();
    let (_model, diags) = compiler
        .compile(mu::Source::from("Unknown(radius = 4mm);"))
        .expect("A model, despite errors");

    assert!(diags.has_errors(), "{diags:?}");
}

/// Built-in transforms and operations of `__mu::ops`.
#[test]
fn compile_ops() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from(
            r#"
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::scale(x = 2.0, y = 3.0, z = 1.0);
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::orient(x = 0.0, y = 1.0, z = 0.0);
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::hull();
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::buffer(distance = 0.5mm);
            __mu::geo2d::Circle(radius = 1mm).__mu::ops::revolve(angle = 360°);
            "#,
        ))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_ops", display(&model));
}

/// Built-in primitives of `__mu::geo2d`.
#[test]
fn compile_geo2d() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from(
            r#"
            __mu::geo2d::Ngon(n = 6, radius = 10mm);
            __mu::geo2d::Sector(radius = 10mm, start = 0°, end = 90°);
            __mu::geo2d::RoundedRect(
                width = 20mm, height = 10mm, x = 0mm, y = 0mm,
                radius_bottom_left = 1mm, radius_bottom_right = 1mm,
                radius_top_left = 2mm, radius_top_right = 2mm,
            );
            __mu::geo2d::Line(x0 = 0.0, y0 = 0.0, x1 = 10.0, y1 = 5.0);
            __mu::geo2d::InvoluteGearProfile(module = 1mm, teeth = 12, pressure_angle = 20°);
            __mu::geo2d::SinusoidalGearProfile(module = 1mm, teeth = 12, roundness = 0.5, stretch = 0.5);
            __mu::geo2d::Text(height = 5mm, text = "µcad", font_file = "");
            "#,
        ))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_geo2d", display(&model));
}

/// Output type of the first model in a compiled source.
fn output_type(model: &mu::ModelTree) -> mu::types::ModelType {
    use mu::types::model::NodeExt;
    model.root().deduce_output_type()
}

/// Built-in primitives of `__mu::geo3d`.
#[test]
fn compile_geo3d() {
    let mut compiler = compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from("__mu::geo3d::Sphere(radius = 10mm);"))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(output_type(&model), mu::types::ModelType::Geometry3D);
    insta::assert_snapshot!("compile_geo3d", display(&model));
}

/// Like `std::geo3d::Torus` and `std::geo3d::Cylinder`: 3D models made of 2D primitives.
#[test]
fn compile_geo3d_from_2d() {
    let mut compiler = compiler();
    for source in [
        // Torus(major_radius = 45mm, minor_radius = 5mm)
        "__mu::geo2d::Circle(radius = 5mm).__mu::ops::translate(x = 45mm, y = 0mm, z = 0mm).__mu::ops::revolve(angle = 360°);",
        // Cylinder(radius = 10mm, height = 20mm)
        "__mu::geo2d::Circle(radius = 10mm).__mu::ops::extrude(height = 20mm, scale_x = 1.0, scale_y = 1.0, twist = 0°).__mu::ops::translate(x = 0mm, y = 0mm, z = -10mm);",
    ] {
        let (model, diags) = compiler.compile(mu::Source::from(source)).expect("A model");

        assert!(!diags.has_errors(), "{diags:?}");
        assert_eq!(
            output_type(&model),
            mu::types::ModelType::Geometry3D,
            "{source}"
        );
    }
}

/// A compiler can be used again, the main library of the previous run is replaced.
#[test]
fn compile_twice() {
    let mut compiler = compiler();
    let (first, _) = compiler.compile(mu::Source::from(DISC)).expect("A model");
    let (second, diags) = compiler.compile(mu::Source::from(DISC)).expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(display(&first), display(&second));
}

/*
/// A source using the standard library.
#[test]
fn compile_with_std() {
    let mut compiler = std_compiler();
    let (model, diags) = compiler
        .compile(mu::Source::from("std::geo2d::Circle(radius = 4mm);"))
        .expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_with_std", display(&model));
}
*/
