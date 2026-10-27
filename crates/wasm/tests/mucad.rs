// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests of the WebAssembly interface, which run natively.

use mucad_wasm::{Mucad, Severity};

const CSG_CUBE: &str = include_str!("../../../examples/csg_cube.mu");
const DRILL_PLATE: &str = include_str!("../../../examples/drill_plate.mu");

/// A compiled toolchain, which panics with the diagnostics on errors.
fn compiled(code: &str, model_type: &str) -> Mucad {
    let mut mucad = Mucad::new();
    let result = mucad.compile(code);
    assert!(result.success(), "{}", result.report());
    assert_eq!(result.model_type(), model_type);
    mucad
}

/// A 3D model is rendered into a triangle mesh and exported as STL.
#[test]
fn render_3d() {
    let mut mucad = compiled(CSG_CUBE, "3d");

    let meshes = mucad.render_meshes().expect("Meshes");
    let [mesh] = &meshes[..] else {
        panic!("A mesh with the default color");
    };
    assert_eq!(mesh.color(), None);
    // The JavaScript arrays of `positions` and `indices` are not available natively.
    let triangle_mesh = mesh.triangle_mesh();
    assert!(!triangle_mesh.indices().is_empty());
    assert_eq!(triangle_mesh.indices().len() % 3, 0);
    assert_eq!(triangle_mesh.coordinates().len() % 3, 0);
    let vertices = (triangle_mesh.coordinates().len() / 3) as u32;
    assert!(
        triangle_mesh
            .indices()
            .iter()
            .all(|index| *index < vertices)
    );

    // Binary STL: 80 bytes header, the number of triangles and 50 bytes per triangle.
    let stl = mucad.export_stl().expect("An STL");
    let triangles = u32::from_le_bytes(stl[80..84].try_into().expect("4 bytes"));
    assert_eq!(stl.len(), 84 + 50 * triangles as usize);
}

/// A 2D model is rendered into polygons, lines and triangles and exported as SVG.
#[test]
fn render_2d() {
    let mut mucad = compiled(DRILL_PLATE, "2d");

    let sketches = mucad.render_sketches().expect("Sketches");
    let [sketch] = &sketches[..] else {
        panic!("A sketch with the default color");
    };

    // One plate with four holes.
    let polygons = sketch.polygons();
    assert_eq!(polygons.polygons(), [0, 5]);
    assert_eq!(polygons.rings().len(), 6);
    assert_eq!(
        *polygons.rings().last().expect("End index") as usize,
        polygons.coordinates().len() / 2
    );

    let lines = sketch.lines();
    assert!(!lines.positions().is_empty());
    assert_eq!(lines.positions().len() % 4, 0);

    let triangles = sketch.triangles();
    let triangles = triangles.triangle_mesh();
    assert!(!triangles.indices().is_empty());
    assert!(
        triangles
            .coordinates()
            .chunks(3)
            .all(|position| position[2] == 0.0)
    );

    let svg = mucad.export_svg().expect("An SVG");
    assert!(svg.contains("<svg"), "{svg}");
}

/// Models with a color attribute are rendered into one mesh or sketch for each color.
#[test]
fn render_colors() {
    let mut mucad = compiled(
        r#"
        use std::geo3d::*;
        #[color = "red"]
        Cube(1mm);
        Sphere(r = 1mm).translate(x = 5mm);
        #[color = "red"]
        Cube(1mm).translate(x = 10mm);
        "#,
        "3d",
    );
    let colors: Vec<_> = mucad
        .render_meshes()
        .expect("Meshes")
        .iter()
        .map(|mesh| mesh.color())
        .collect();
    assert_eq!(colors, [Some(vec![1.0, 0.0, 0.0, 1.0]), None]);

    let mut mucad = compiled("#[color = \"#0000FF\"]\nstd::geo2d::Circle(r = 1mm);", "2d");
    let sketches = mucad.render_sketches().expect("Sketches");
    assert_eq!(sketches[0].color(), Some(vec![0.0, 0.0, 1.0, 1.0]));

    let svg = mucad.export_svg().expect("An SVG");
    assert!(svg.contains("#0000FF"), "{svg}");
}

/// Errors are reported as diagnostics, a model is only kept without errors.
#[test]
fn compile_errors() {
    let mut mucad = Mucad::new();
    let result = mucad.compile("Unknown();");
    assert!(!result.success());
    assert!(
        result.report().contains("Unresolved symbol"),
        "{}",
        result.report()
    );
    assert_eq!(result.model_type(), "none");
    assert!(mucad.render_meshes().is_err());
    assert!(mucad.export_svg().is_err());
}

/// Diagnostics have a range with lines and UTF-16 columns from 1, e.g. for editor markers.
#[test]
fn diagnostics() {
    let mut mucad = Mucad::new();
    // `µ` and `🚀` are 2 and 4 bytes, but 1 and 2 UTF-16 code units.
    let result = mucad.compile("// µ\nx = \"🚀\"; Unknown();");
    assert!(!result.success());

    let diagnostics = result.diagnostics();
    let unresolved = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message().contains("Unresolved symbol"))
        .expect("An unresolved symbol");
    assert_eq!(unresolved.severity(), Severity::Error);
    assert_eq!(
        (
            unresolved.start_line_no(),
            unresolved.start_column(),
            unresolved.end_line_no(),
            unresolved.end_column()
        ),
        (2, 11, 2, 18)
    );
}

/// A shared link is decoded into its formatted code, invalid links fail.
#[test]
fn decode_link() {
    assert_eq!(
        Mucad::decode_link("https://play.mucad.org/#wrVBRAAEQ2lyY2xlKHIgPSAxMG1tKTsKAw"),
        Ok("Circle(r = 10mm);\n".to_string())
    );
    assert_eq!(
        Mucad::decode_link("wrVBRAAEQ2lyY2xlKHIgPSAxMG1tKTsKAw"),
        Ok("Circle(r = 10mm);\n".to_string())
    );
    assert!(Mucad::decode_link("#not base64").is_err());
}

/// Code is encoded into a link, also without comments.
#[test]
fn encode_link() {
    let link = Mucad::encode_link("Circle(r = 10mm);\n", false).expect("A link");
    assert_eq!(
        link,
        "https://play.mucad.org/#wrVBRAAEQ2lyY2xlKHIgPSAxMG1tKTsKAw"
    );
    let stripped = Mucad::encode_link("// c\nCircle(r = 10mm);\n", true).expect("A link");
    assert_eq!(stripped, link);
    assert!(Mucad::encode_link(&"a".repeat(3000), false).is_err());
}

/// The QR code of a link is square.
#[test]
fn qr_code() {
    let modules = Mucad::qr_code("https://play.mucad.org/#wrVBRAAEQ2lyY2xlKHIgPSAxMG1tKTsKAw")
        .expect("A QR code");
    let width = modules.len().isqrt();
    assert_eq!(width * width, modules.len());
    // A QR code of version 4 for 58 bytes with a finder pattern in the corner.
    assert_eq!(width, 33);
    assert_eq!(modules[..7], [1; 7]);
}

/// The names in front of a position are completed with the symbols of the last compilation,
/// also while typing, after characters outside of ASCII.
#[test]
fn complete() {
    let mut mucad = Mucad::new();
    let code = "/// A disc.\nsketch Disc() {}\ns = \"µ🚀\"; Disc();\n";
    assert!(mucad.compile(code).success());

    // The changed code is not compiled, its last line is `s = "µ🚀"; Di`.
    let typed = "/// A disc.\nsketch Disc() {}\ns = \"µ🚀\"; Di";
    let completions = mucad.complete(typed, 3, 13);
    let disc = completions
        .iter()
        .find(|completion| completion.name() == "Disc")
        .expect("Disc");
    assert_eq!(disc.kind(), "sketch");
    assert_eq!(disc.doc().as_deref(), Some("A disc."));
    assert!(
        completions
            .iter()
            .any(|completion| completion.name() == "std")
    );

    // The symbols of a module after `::`.
    let completions = mucad.complete("std::geo2d::Ci", 1, 15);
    let names: Vec<_> = completions
        .iter()
        .map(|completion| completion.name())
        .collect();
    assert!(names.contains(&"Circle".to_string()), "{names:?}");
    assert!(!names.contains(&"Disc".to_string()), "{names:?}");

    // Numbers are not completed.
    assert!(mucad.complete("x = 12", 1, 7).is_empty());
}
