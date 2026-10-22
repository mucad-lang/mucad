// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Types of functions.

mod common;

use mucad_lang_base::Source;
use mucad_lang_lower::ir;
use mucad_lang_types::{Ty, Type};

/// The functions of `code` with their name.
fn functions(code: &str) -> Vec<(String, ir::Function)> {
    let (ir, issues) = common::ir_from_source(&Source::from(code)).expect("An IR");
    assert!(issues.is_empty(), "{issues:?}");
    ir.tree
        .root()
        .descendants()
        .filter_map(|node| match (&node.meta.name, &node.def) {
            (Some(name), ir::Def::Function(function)) => Some((name.to_string(), function.clone())),
            _ => None,
        })
        .collect()
}

/// Functions return their declared type, without one their result is known when calling.
#[test]
fn function_types() {
    let functions = functions(
        "fn area(width: Length, height: Length) -> Area { return width * height; }\n\
         fn twice(x = 1mm) -> Length { return 2 * x; }\n\
         fn any(x: Any) { return x; }",
    );
    let function = |name: &str| {
        &functions
            .iter()
            .find(|(function, _)| function == name)
            .unwrap_or_else(|| panic!("A function {name}"))
            .1
    };

    assert_eq!(
        function("area").return_type(),
        Some(Type::Quantity(mucad_lang_types::QuantityType::Area))
    );
    assert_eq!(
        function("area").ty().to_string(),
        "(width: Length, height: Length) -> Area"
    );

    // A parameter without type has the type of its default value.
    assert_eq!(function("twice").ty().to_string(), "(x: Length) -> Length");

    assert_eq!(function("any").return_type(), Some(Type::Any));
}
