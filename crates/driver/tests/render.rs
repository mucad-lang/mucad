// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rendering of compiled sources with a render cache, e.g. in the viewer.

use mucad_core::Vec3;
use mucad_driver::{Compiler, prelude as mu};

/// A compiler and a renderer with a render cache, which are kept between compilations.
struct Session {
    compiler: Compiler,
    renderer: mu::render::Renderer,
}

impl Session {
    fn new() -> Self {
        Self {
            compiler: Compiler::from_config(&mu::DriverConfig {
                search_paths: vec!["../std/lib".into()],
                ..Default::default()
            }),
            renderer: mu::render::Renderer::new()
                .with_cache(mu::SharedMut::new(mu::render::RenderCache::new())),
        }
    }

    fn render(&mut self, code: &str) -> mu::render::GeometryTree {
        let (model, diags) = self.compiler.compile(code).expect("A model");
        assert!(!diags.has_errors(), "{diags:?}");
        let (tree, issues) = self.renderer.render(&model).expect("A geometry tree");
        assert!(!issues.has_errors(), "{issues:?}");
        tree
    }
}

/// A built-in without render hook is an error, but rendering continues with the other models.
#[test]
fn render_missing_hook() {
    let mut session = Session::new();
    let (model, _) = session
        .compiler
        .compile("__mu::geo2d::Ngon(n = 6, radius = 1mm);\nCircle(radius = 2mm);")
        .expect("A model");
    let (tree, issues) = session.renderer.render(&model).expect("A geometry tree");

    let errors: Vec<_> = issues.iter().map(|issue| issue.to_string()).collect();
    assert_eq!(errors, ["Built-in `__mu::geo2d::Ngon` cannot be rendered"]);
    assert!(
        (tree.bounds().max.x - 2.0).abs() < 1e-6,
        "{:?}",
        tree.bounds()
    );
}

/// A changed source is rendered again, although its model tree has the same structure.
#[test]
fn render_changed_source() {
    let mut session = Session::new();
    let small = session.render("Circle(radius = 1mm);");
    let large = session.render("Circle(radius = 5mm);");

    assert_ne!(small.root().hash, large.root().hash);
    assert!((small.bounds().max.x - 1.0).abs() < 1e-6);
    assert!((large.bounds().max.x - 5.0).abs() < 1e-6);
}

/// An unchanged source has the same hashes, so its geometry is taken from the render cache.
#[test]
fn render_unchanged_source() {
    let mut session = Session::new();
    let first = session.render("Circle(radius = 1mm);");
    let second = session.render("Circle(radius = 1mm);");

    assert_eq!(first.root().hash, second.root().hash);
    assert_eq!(first.bounds().max, second.bounds().max);
}

/// The hash of a node contains the hashes of its children.
#[test]
fn render_changed_child() {
    let mut session = Session::new();
    let first = session.render("{ Circle(radius = 1mm); Rect(2mm); }");
    let second = session.render("{ Circle(radius = 1mm); Rect(3mm); }");

    assert_ne!(first.root().hash, second.root().hash);
    assert!(
        (second.bounds().max.x - 1.5).abs() < 1e-6,
        "{:?}",
        second.bounds()
    );
}

/// The bounds of the rendered geometry of `code`, which uses `std::ops`.
fn ops_bounds(code: &str) -> mucad_core::Bounds3D {
    let tree = Session::new().render(&format!("use std::ops::*;\n{code}"));
    tree.bounds()
}

fn assert_bounds(code: &str, min: impl Into<Vec3>, max: impl Into<Vec3>) {
    let bounds = ops_bounds(code);
    let (actual_min, actual_max) = bounds.min_max();
    let close = |a: &Vec3, b: Vec3| {
        (a.x - b[0]).abs() < 1e-6 && (a.y - b[1]).abs() < 1e-6 && (a.z - b[2]).abs() < 1e-6
    };
    assert!(
        close(actual_min, min.into()) && close(actual_max, max.into()),
        "{code}: {actual_min:?} {actual_max:?}"
    );
}

/// Models are aligned one after another with spacing.
#[test]
fn render_align() {
    assert_bounds(
        "Circle(radius = [1, 2]mm).align(std::math::X, 1mm);",
        [0.0, -2.0, 0.0],
        [7.0, 2.0, 0.0],
    );
}

/// Mirror keeps the original, reflect only keeps the reflection.
#[test]
fn render_mirror_reflect() {
    assert_bounds(
        "Circle(radius = 1mm).translate(x = 3mm).mirror(n = std::math::X);",
        [-4.0, -1.0, 0.0],
        [4.0, 1.0, 0.0],
    );
    assert_bounds(
        "Circle(radius = 1mm).translate(x = 3mm).reflect(n = std::math::X);",
        [-4.0, -1.0, 0.0],
        [-2.0, 1.0, 0.0],
    );
    assert_bounds(
        "std::geo3d::Cube(2mm).translate(z = 3mm).mirror(n = std::math::Z);",
        [-1.0, -1.0, -4.0],
        [1.0, 1.0, 4.0],
    );
}

/// Copies of a model are distributed in the cells of a grid.
#[test]
fn render_multiply_distribute_grid() {
    let tree = Session::new().render("Circle(radius = 1mm).std::ops::multiply(3);");
    assert_eq!(tree.root().outputs.len(), 3);

    assert_bounds(
        "Circle(radius = 1mm).multiply(4).distribute_grid(size = 10mm, rows = 2, columns = 2);",
        [-3.5, -3.5, 0.0],
        [3.5, 3.5, 0.0],
    );
}

/// A profile is turned around the axis at a distance and moved along the axis.
#[test]
fn render_spiralize() {
    assert_bounds(
        "Circle(radius = 1mm).spiralize(inner_radius = 5mm, outer_radius = 5mm, angle = 360°, height = 10mm);",
        [-6.0, -6.0, -1.0],
        [6.0, 6.0, 11.0],
    );
    // The distance from the axis grows from the inner to the outer radius:
    // after half a turn, the profile is at a distance of 4mm on the other side.
    let bounds = ops_bounds(
        "Circle(radius = 1mm).spiralize(inner_radius = 2mm, outer_radius = 4mm, angle = 180°, height = 4mm);",
    );
    let (min, max) = bounds.min_max();
    assert!((min.x + 5.0).abs() < 1e-6, "{min:?}");
    assert!(
        (min.z + 1.0).abs() < 1e-6 && (max.z - 5.0).abs() < 1e-6,
        "{min:?} {max:?}"
    );
}
