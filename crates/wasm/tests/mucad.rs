// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests of the WebAssembly interface, which run natively.

use mucad_wasm::Mucad;

const CSG_CUBE: &str = include_str!("../../../examples/csg_cube.mu");
const DRILL_PLATE: &str = include_str!("../../../examples/drill_plate.mu");

/// A compiled toolchain, which panics with the diagnostics on errors.
fn compiled(code: &str, model_type: &str) -> Mucad {
    let mut mucad = Mucad::new();
    let result = mucad.compile(code);
    assert!(result.success(), "{}", result.diagnostics());
    assert_eq!(result.model_type(), model_type);
    mucad
}

/// A 3D model is rendered into a triangle mesh and exported as STL.
#[test]
fn render_3d() {
    let mut mucad = compiled(CSG_CUBE, "3d");

    let mesh = mucad.render_mesh().expect("A mesh");
    assert!(!mesh.indices().is_empty());
    assert_eq!(mesh.indices().len() % 3, 0);
    assert_eq!(mesh.positions().len() % 3, 0);
    let vertices = (mesh.positions().len() / 3) as u32;
    assert!(mesh.indices().iter().all(|index| *index < vertices));

    // Binary STL: 80 bytes header, the number of triangles and 50 bytes per triangle.
    let stl = mucad.export_stl().expect("An STL");
    let triangles = u32::from_le_bytes(stl[80..84].try_into().expect("4 bytes"));
    assert_eq!(stl.len(), 84 + 50 * triangles as usize);
}

/// A 2D model is rendered into polygons, lines and triangles and exported as SVG.
#[test]
fn render_2d() {
    let mut mucad = compiled(DRILL_PLATE, "2d");

    // One plate with four holes.
    let polygons = mucad.render_polygons().expect("Polygons");
    assert_eq!(polygons.polygons(), [0, 5]);
    assert_eq!(polygons.rings().len(), 6);
    assert_eq!(
        *polygons.rings().last().expect("End index") as usize,
        polygons.coordinates().len() / 2
    );

    let lines = mucad.render_lines().expect("Lines");
    assert!(!lines.positions().is_empty());
    assert_eq!(lines.positions().len() % 4, 0);

    let triangles = mucad.render_triangles().expect("Triangles");
    assert!(!triangles.indices().is_empty());
    assert!(
        triangles
            .positions()
            .chunks(3)
            .all(|position| position[2] == 0.0)
    );

    let svg = mucad.export_svg().expect("An SVG");
    assert!(svg.contains("<svg"), "{svg}");
}

/// Errors are reported as diagnostics, a model is only kept without errors.
#[test]
fn compile_errors() {
    let mut mucad = Mucad::new();
    let result = mucad.compile("Unknown();");
    assert!(!result.success());
    assert!(
        result.diagnostics().contains("Unresolved symbol"),
        "{}",
        result.diagnostics()
    );
    assert_eq!(result.model_type(), "none");
    assert!(mucad.render_mesh().is_err());
    assert!(mucad.export_svg().is_err());
}
