// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::{BuiltinEvalContext, mu::*};
use mucad_macros::test_builtin_fn;

use mucad_lang_types::{Value, arguments, list, tuple};

// --- Arithmetic Operators ---

#[test]
#[test_builtin_fn(core::add(lhs = 5, rhs = 3) == 8)]
#[test_builtin_fn(core::add(lhs = -2, rhs = 4) == 2)]
fn add() {}

#[test]
#[test_builtin_fn(core::sub(lhs = 10, rhs = 4) == 6)]
#[test_builtin_fn(core::sub(lhs = 2, rhs = 5) == -3)]
fn sub() {}

#[test]
#[test_builtin_fn(core::mul(lhs = 4, rhs = 3) == 12)]
#[test_builtin_fn(core::mul(lhs = -3, rhs = 3) == -9)]
fn mul() {}

#[test]
#[test_builtin_fn(core::div(lhs = 12, rhs = 3) == 4.0)]
#[test_builtin_fn(core::div(lhs = 7, rhs = 2) == 3.5)]
fn div() {}

// --- Unary Operators ---

#[test]
#[test_builtin_fn(core::neg(rhs = 5) == -5)]
#[test_builtin_fn(core::neg(rhs = -3) == 3)]
fn neg() {}

#[test]
#[test_builtin_fn(core::plus(rhs = 5) == 5)]
#[test_builtin_fn(core::plus(rhs = -3) == -3)]
fn plus() {}

#[test]
#[test_builtin_fn(core::not(rhs = true) == false)]
#[test_builtin_fn(core::not(rhs = false) == true)]
fn not() {}

// --- Comparison Operators ---

#[test]
#[test_builtin_fn(core::gt(lhs = 3, rhs = 5) == false)]
#[test_builtin_fn(core::gt(lhs = 5, rhs = 3) == true)]
#[test_builtin_fn(core::gt(lhs = 4, rhs = 4) == false)]
fn gt() {}

#[test]
#[test_builtin_fn(core::lt(lhs = 3, rhs = 5) == true)]
#[test_builtin_fn(core::lt(lhs = 5, rhs = 3) == false)]
#[test_builtin_fn(core::lt(lhs = 4, rhs = 4) == false)]
fn lt() {}

#[test]
#[test_builtin_fn(core::ge(lhs = 5, rhs = 3) == true)]
#[test_builtin_fn(core::ge(lhs = 4, rhs = 4) == true)]
#[test_builtin_fn(core::ge(lhs = 3, rhs = 5) == false)]
fn ge() {}

#[test]
#[test_builtin_fn(core::le(lhs = 3, rhs = 5) == true)]
#[test_builtin_fn(core::le(lhs = 4, rhs = 4) == true)]
#[test_builtin_fn(core::le(lhs = 5, rhs = 3) == false)]
fn le() {}

#[test]
#[test_builtin_fn(core::eq(lhs = 4, rhs = 4) == true)]
#[test_builtin_fn(core::eq(lhs = 4, rhs = 5) == false)]
fn eq() {}

#[test]
#[test_builtin_fn(core::not_equal(lhs = 4, rhs = 5) == true)]
#[test_builtin_fn(core::not_equal(lhs = 4, rhs = 4) == false)]
fn not_equal() {}

// --- Logical / Bitwise Operators ---

#[test]
#[test_builtin_fn(core::and(lhs = true, rhs = true) == true)]
#[test_builtin_fn(core::and(lhs = true, rhs = false) == false)]
fn and() {}

#[test]
#[test_builtin_fn(core::or(lhs = true, rhs = false) == true)]
#[test_builtin_fn(core::or(lhs = false, rhs = false) == false)]
fn or() {}

// --- Data Structure Access ---
// --- Data Structure Access & Construction ---

#[test]
#[test_builtin_fn(core::list_access(lhs = list![10, 20, 30], index = 1) == 20)]
#[test_builtin_fn(core::list_access(lhs = list![10, 20, 30], index = 0) == 10)]
fn list_access() {}

#[test]
#[test_builtin_fn(core::member_access(lhs = tuple!(a = 42, b = "hello"), name = "a") == 42)]
#[test_builtin_fn(core::member_access(lhs = tuple!(a = 42, b = "hello"), name = "b") == "hello")]
fn member_access() {}

#[test]
fn list() {
    let mut ctx = BuiltinEvalContext::default();
    match core::list(arguments!(1, 2, 3, 4), &mut ctx) {
        Ok(Value::List(a)) => assert_eq!(a.as_ref(), &list![1, 2, 3, 4]),
        _ => panic!("Expected list input"),
    }
}

#[test]
#[test_builtin_fn(core::tuple(x = 10, y = 20) == tuple!(x = 10, y = 20))]
fn tuple() {}

#[test]
#[test_builtin_fn(core::range(start = 1, end = 4) == list![1, 2, 3, 4])]
#[test_builtin_fn(core::range(start = 0, end = 0) == list![0])]
fn range() {}

// --- String Formatting ---

#[test]
fn format() {
    let mut ctx = BuiltinEvalContext::default();
    match core::format(arguments!("Hello, ", "World!", " ", 2026), &mut ctx) {
        Ok(Value::String(s)) => assert_eq!(s.as_str(), "Hello, World! 2026"),
        _ => panic!("Formatting failed"),
    }
}

#[test]
#[test_builtin_fn(core::format_spec(expr = 3.14159, width = 8, precision = 2) == "    3.14")]
#[test_builtin_fn(core::format_spec(expr = 42, width = 5, precision = -1) == "   42")]
#[test_builtin_fn(core::format_spec(expr = 3.14159, width = -1, precision = 3) == "3.142")]
fn format_spec() {}
