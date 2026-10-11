// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the high-level compiler API.

use microcad_driver::{Compiler, prelude as mu};
use mu::base::DisplayWithCtx;

/// A sketch and an operation, called as method on the sketch.
const DISC: &str = r#"
sketch Disc(radius: Length) {
    __mu::geo2d::Circle(radius = radius);
}

op Raise(height: Length) {
    __mu::ops::extrude(self = @input, height = height);
}

Disc(radius = 4mm).Raise(height = 2mm);
"#;

fn display(model: &mu::ModelTree) -> String {
    model.to_string_with_ctx(&mu::builtin::BuiltinLibrary::new())
}

#[test]
fn compile_source() {
    let mut compiler = Compiler::new();
    let (model, diags) = compiler.compile(mu::Source::from(DISC)).expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    insta::assert_snapshot!("compile_source", display(&model));
}

#[test]
fn compile_unresolved_symbol() {
    let mut compiler = Compiler::new();
    let (_model, diags) = compiler
        .compile(mu::Source::from("Unknown(radius = 4mm);"))
        .expect("A model, despite errors");

    assert!(diags.has_errors(), "{diags:?}");
}

/// A compiler can be used again, the main library of the previous run is replaced.
#[test]
fn compile_twice() {
    let mut compiler = Compiler::new();
    let (first, _) = compiler.compile(mu::Source::from(DISC)).expect("A model");
    let (second, diags) = compiler.compile(mu::Source::from(DISC)).expect("A model");

    assert!(!diags.has_errors(), "{diags:?}");
    assert_eq!(display(&first), display(&second));
}
