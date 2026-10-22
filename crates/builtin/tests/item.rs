// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Types of built-in items.

use mucad_builtin::{BuiltinId, BuiltinItem, BuiltinLibrary};
use mucad_lang_types::{MatrixType, ModelType, Type};

fn item(name: &str) -> &'static BuiltinItem {
    BuiltinLibrary::new()
        .get(BuiltinId::from_name(name))
        .unwrap_or_else(|| panic!("A built-in {name}"))
}

/// Functions, primitives and operations return the type of their signature.
#[test]
fn return_types() {
    for (name, ty) in [
        ("__mu::core::gt", Some(Type::Bool)),
        ("__mu::core::format", Some(Type::String)),
        ("__mu::core::add", Some(Type::Any)),
        (
            "__mu::math::rotate_xyz",
            Some(Type::Matrix(MatrixType::new(3, 3))),
        ),
        (
            "__mu::geo2d::Circle",
            Some(Type::Model(ModelType::Geometry2D)),
        ),
        (
            "__mu::ops::extrude",
            Some(Type::Model(ModelType::Geometry3D)),
        ),
        // Functions with side effects only return nothing.
        ("__mu::debug::print", None),
        // Constants cannot be called.
        ("__mu::math::PI", None),
    ] {
        assert_eq!(item(name).return_type(), ty, "{name}");
    }
}

/// Constants have the type of their value, callables their function type, modules none.
#[test]
fn types() {
    assert_eq!(item("__mu::math::PI").ty(), Some(Type::scalar()));

    let Some(Type::Function(function)) = item("__mu::core::gt").ty() else {
        panic!("A function type");
    };
    assert_eq!(function.return_ty.as_deref(), Some(&Type::Bool));

    assert!(matches!(
        item("__mu::ops::extrude").ty(),
        Some(Type::Function(_))
    ));
    assert_eq!(item("__mu::math").ty(), None);
}
