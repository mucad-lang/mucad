// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for rendering model trees.

use mucad_builtin::{BuiltinEvalContext, BuiltinPrimitiveCall, mu};
use mucad_lang_types::{Model, ModelTree, Value, arguments, model::Element};
use mucad_render::Renderer;

fn circle(r: f64) -> Model {
    let mut ctx = BuiltinEvalContext::default();
    mu::geo2d::Circle::call(arguments!(radius = Value::mm(r)), &mut ctx).expect("No error")
}

/// Circle(42mm);
#[test]
fn render_circle() {
    let circle = circle(42.0);
    let mut model_tree = ModelTree::new(circle);
    model_tree.compute_hashes();

    let geometry = Renderer::new()
        .render(&model_tree)
        .expect("No render errors")
        .0;

    insta::assert_snapshot!("render_circle", geometry);
}

/// Circle(42mm) - Circle(23mm);
#[test]
fn render_subtract() {
    let mut ctx = BuiltinEvalContext::default();

    // {
    //     Circle(42mm);
    //     Circle(23mm);
    // }
    let mut group = ModelTree::new(Element::Group);
    group.append(circle(42.0));
    group.append(circle(23.0));
    let mut diff = mu::ops::subtract(arguments!(self = group), &mut ctx).expect("No error");
    diff.compute_hashes();

    let geometry = Renderer::new().render(&diff).expect("No render errors").0;

    insta::assert_snapshot!("render_difference", geometry);
}

/// Circle(42mm).extrude(23mm);
#[test]
fn render_extrude() {
    let mut ctx = BuiltinEvalContext::default();

    let mut extrude = mu::ops::extrude(
        arguments!(
            self = circle(42.0),
            height = Value::mm(23.0),
            scale_x = 1.0,
            scale_y = 1.0,
            twist = Value::deg(0.0)
        ),
        &mut ctx,
    )
    .expect("No error");
    extrude.compute_hashes();

    let geometry = Renderer::new()
        .render(&extrude)
        .expect("No render errors")
        .0;

    insta::assert_snapshot!("render_extrude", geometry);
}
