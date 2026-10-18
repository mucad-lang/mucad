// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_types::{Value, ValueResult};

fn check(result: ValueResult, value: impl Into<Value>) {
    let result = result.expect("error result");
    assert_eq!(result, value.into());
}

#[test]
fn test_value_integer() {
    let u = || Value::from(2);
    let v = || Value::from(5);
    let w = || Value::from(5.0);

    // symmetric operations
    check(u() + v(), 2 + 5);
    check(u() - v(), 2 - 5);
    check(u() * v(), 2 * 5);
    check(u() / v(), 2.0 / 5.0);
    check(-u(), -2);

    // asymmetric operations
    check(u() + w(), 2.0 + 5.0);
    check(u() - w(), 2.0 - 5.0);
    check(u() * w(), 2.0 * 5.0);
    check(u() / w(), 2.0 / 5.0);
}

#[test]
fn test_value_scalar() {
    let u = || Value::from(2.0);
    let v = || Value::from(5.0);
    let w = || Value::from(5);

    // symmetric operations
    check(u() + v(), 2.0 + 5.0);
    check(u() - v(), 2.0 - 5.0);
    check(u() * v(), 2.0 * 5.0);
    check(u() / v(), 2.0 / 5.0);
    check(-u(), -2.0);

    // asymmetric operations
    check(u() + w(), 2.0 + 5.0);
    check(u() - w(), 2.0 - 5.0);
    check(u() * w(), 2.0 * 5.0);
    check(u() / w(), 2.0 / 5.0);
}
