// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rendering of compiled sources with a render cache, e.g. in the viewer.

use microcad_driver::{Compiler, prelude as mu};

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
        self.renderer.render(&model).expect("A geometry tree")
    }
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
