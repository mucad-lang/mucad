// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for evaluating functions.

use mucad_builtin::__mu;
use mucad_lang_base::{SrcRef, SymbolId, boxed};
use mucad_lang_eval::{Callable, EvalContext};
use mucad_lang_resolve::{
    call_builtin, expr,
    library::symbol::{
        self, ConstantValue, Function, FunctionExpression, FunctionStatement, Parameter, function,
    },
};
use mucad_lang_types::{ArgumentValueList, Type, Value, argument_value};
use mucad_macros::parameter_list;

fn scope(a: impl Iterator<Item = FunctionStatement>) -> function::Scope {
    function::Scope {
        statements: boxed(a),
        src_ref: SrcRef::none(),
    }
}

fn assert_eval(
    f: &impl Callable,
    args: impl IntoIterator<Item = mucad_lang_types::ArgumentValue>,
    expected: impl Into<Value>,
) {
    let mut context = EvalContext::new();
    let result = f
        .call(&ArgumentValueList::from_iter(args), &mut context)
        .expect("No eval error");

    assert_eq!(result, expected.into());
}

#[test]
fn return_a() {
    let f = function::Function::new(parameter_list!(a: Integer))
        .with_return_type(Type::Integer)
        .with_statements([function::ReturnStatement {
            expr: Some(expr!(a).into()),
            keyword_src_ref: SrcRef::none(),
            src_ref: SrcRef::none(),
        }
        .into()]);

    assert_eval(&f, [argument_value!(a = 2)], 2);
}

/// Test a function `f(a: Integer, b: Integer) -> Integer` that return the sum of `a` and `b`.
#[test]
fn add() {
    let f = function::Function::new(parameter_list!(a: Integer, b: Integer))
        .with_return_type(Type::Integer)
        .with_statements([FunctionStatement::Tail(
            call_builtin!(core::add(lhs = expr!(a), rhs = expr!(b))).into(),
        )]);

    // Call the function with `f(a = 1, b = 3)`
    assert_eval(&f, [argument_value!(a = 1), argument_value!(b = 3)], 4);
}

#[test]
fn if_a_greater_than() {
    let f = Function::new(parameter_list!(a: Integer, b: Integer))
        .with_return_type(Type::Integer)
        .with_statements([function::FunctionIf {
            src_ref: SrcRef::none(),
            if_ref: SrcRef::none(),
            cond: FunctionExpression::Call(call_builtin!(
                core::gt(lhs = expr!(a), rhs = expr!(b),)
            ))
            .into(),
            body: scope([FunctionStatement::Tail(ConstantValue::new(2).into())].into_iter()).into(),
            else_ref: None,
            body_else: Some(
                scope([FunctionStatement::Tail(ConstantValue::new(4).into())].into_iter()).into(),
            ),
            next_if: None,
        }
        .into()]);

    assert_eval(&f, [argument_value!(a = 1), argument_value!(b = 3)], 4);
    assert_eval(&f, [argument_value!(a = 3), argument_value!(b = 1)], 2);
}
