// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_types::{Quantity, QuantityType, Scalar, Value};

#[test]
fn test_value_from_str_literals() {
    let cases: Vec<(&str, Value)> = vec![
        (
            "0.1mm",
            Quantity::new(Scalar::from_num(0.1), QuantityType::Length).into(),
        ), // Expecting a Quantity/Measurement
        (
            "42°",
            Quantity::new(Scalar::from_num(42.0_f64.to_radians()), QuantityType::Angle).into(),
        ), // Expecting a Quantity/Angle
        ("\"Foo\"", "Foo".to_string().into()), // Expecting a String
        ("1", Value::from(1)),                 // Expecting an Integer
        (
            "23.0",
            Quantity::new(Scalar::from_num(23.0), QuantityType::Scalar).into(),
        ), // Expecting a Scalar
    ];

    for (input, output) in cases {
        let result = mucad_driver::value_from_str(input).unwrap();
        assert_eq!(result, output);
    }
}

#[test]
fn test_parse_errors() {
    // Test an invalid literal to ensure your ParseErrors/miette setup works
    let input = "[foo]";
    let result = mucad_driver::value_from_str(input);
    assert!(result.is_err());
}

/// A leading `-` negates quantities and integers, also with whitespace around the literal.
#[test]
fn test_value_from_str_negative() {
    let cases: Vec<(&str, Value)> = vec![
        ("-1.5mm", Value::mm(-1.5)),
        ("-10mm", Value::mm(-10.0)),
        ("-.5", Value::from(-0.5)),
        ("-3", Value::from(-3)),
        (" -3 ", Value::from(-3)),
        ("1e-3", Value::from(0.001)),
    ];
    for (input, output) in cases {
        let result = mucad_driver::value_from_str(input).unwrap();
        assert_eq!(result, output, "{input}");
    }
    // No number after the sign, or a sign in front of other literals.
    for input in ["- 3", "--3", "-true", "-\"a\"", "-", "-x"] {
        assert!(mucad_driver::value_from_str(input).is_err(), "{input}");
    }
}
