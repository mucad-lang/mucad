// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::Identifier;
use mucad_lang_types::{function_type, ty::*};

#[test]
fn test_tuple_type_eq() {
    assert_eq!(TupleType::color(), TupleType::color());
}

#[test]
fn test_tuple_type_match() {
    let args = TupleType {
        named: [
            (Identifier::no_ref("x"), Type::Integer),
            (Identifier::no_ref("y"), Type::List(Box::new(Type::Integer))),
        ]
        .into_iter()
        .collect(),
        positional: Default::default(),
    };
    let params = TupleType {
        named: [
            (Identifier::no_ref("x"), Type::Integer),
            (Identifier::no_ref("y"), Type::Integer),
        ]
        .into_iter()
        .collect(),
        positional: Default::default(),
    };
    assert!(args.matches_replication(&params));
}

#[test]
fn test_common_type() {
    let list = TypeList::new(vec![Type::Integer, Type::Integer]);
    assert_eq!(Some(Type::Integer), list.common_type());

    let list = TypeList::new(vec![Type::Integer, Type::Quantity(QuantityType::Scalar)]);
    assert_eq!(None, list.common_type());

    let list = TypeList::new(Vec::new());
    assert_eq!(None, list.common_type());
}

#[test]
fn type_match() {
    assert!(Type::scalar().matches(&Type::scalar()));
    assert!(!Type::scalar().matches(&Type::Integer));
    assert!(Type::Integer.matches(&Type::scalar()));
    assert!(!Type::scalar().matches(&Type::String));
    assert!(!Type::String.matches(&Type::scalar()));

    assert!(Type::Any.matches(&Type::Any));
    assert!(Type::Any.matches(&Type::scalar()));
    assert!(Type::scalar().matches(&Type::Any));

    assert!(!Type::Invalid.matches(&Type::Any));
}

#[test]
fn function_type_macro() {
    // 1. Variadic with return type
    let ft = function_type!((*) -> Type::Integer);
    assert!(ft.is_variadic());
    assert_eq!(ft.return_ty, Some(Box::new(Type::Integer)));

    // 2. Variadic without return type
    let ft = function_type!((*));
    assert!(ft.is_variadic());
    assert_eq!(ft.return_ty, None);

    // 3. Named parameters with return type
    let ft = function_type!((lhs: Type::Integer, rhs: Type::Integer) -> Type::Integer);
    assert!(!ft.is_variadic());
    assert_eq!(ft.return_ty, Some(Box::new(Type::Integer)));
    assert_eq!(ft.call_ty.parameters.as_ref().unwrap().0.len(), 2);

    // 4. Named parameters without return type
    let ft = function_type!((lhs: Type::Integer, rhs: Type::Integer));
    assert!(!ft.is_variadic());
    assert_eq!(ft.return_ty, None);
}

#[test]
fn parameter_sorting_canonicalization() {
    // Constructor sorts parameters alphabetically by identifier name
    let ft1 = function_type!((a: Type::Integer, b: Type::String) -> Type::Bool);
    let ft2 = function_type!((b: Type::String, a: Type::Integer) -> Type::Bool);

    // Order-independent matching
    assert!(ft1.matches(&ft2));
}

#[test]
fn signature_matching() {
    let sig_a = function_type!((x: Type::Integer, y: Type::scalar()) -> Type::Bool);
    let sig_b = function_type!((x: Type::Integer, y: Type::scalar()) -> Type::Bool);
    let sig_diff_return = function_type!((x: Type::Integer, y: Type::scalar()) -> Type::Integer);
    let sig_diff_param = function_type!((x: Type::Integer, y: Type::String) -> Type::Bool);

    // Identical signatures match
    assert!(sig_a.matches(&sig_b));

    // Mismatched return types do not match
    assert!(!sig_a.matches(&sig_diff_return));

    // Mismatched parameter types do not match
    assert!(!sig_a.matches(&sig_diff_param));
}

#[test]
fn variadic_matching() {
    let variadic = function_type!((*) -> Type::Integer);
    let concrete = function_type!((x: Type::Integer) -> Type::Integer);

    // Variadic accepting concrete
    assert!(variadic.matches(&variadic));

    // Non-variadic to variadic matching checks
    assert!(!variadic.matches(&concrete));
}

#[test]
fn display_formatting() {
    let ft_variadic = function_type!((*) -> Type::Integer);
    assert_eq!(format!("{ft_variadic}"), "(*) -> Integer");

    let ft_params = function_type!((a: Type::Integer) -> Type::Bool);
    assert_eq!(format!("{ft_params}"), "(a: Integer) -> Bool");

    let ft_no_ret = function_type!((a: Type::Integer));
    assert_eq!(format!("{ft_no_ret}"), "(a: Integer)");
}

/// Models of any or undetermined geometry match every model, 2D and 3D models only themselves.
#[test]
fn model_type_match() {
    use mucad_lang_types::ModelType;

    let model = |ty| Type::Model(ty);
    for any in [ModelType::Any, ModelType::NotDetermined] {
        for ty in [ModelType::Geometry2D, ModelType::Geometry3D, ModelType::Any] {
            assert!(model(ty).matches(&model(any)), "{ty} {any}");
            assert!(model(any).matches(&model(ty)), "{any} {ty}");
        }
    }
    assert!(model(ModelType::Geometry2D).matches(&model(ModelType::Geometry2D)));
    assert!(!model(ModelType::Geometry2D).matches(&model(ModelType::Geometry3D)));
    assert!(!model(ModelType::Geometry3D).matches(&model(ModelType::Geometry2D)));
    assert!(!model(ModelType::Any).matches(&Type::length()));
}
