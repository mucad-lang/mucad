// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Find literals in code and step them, e.g. in an editor.

use mucad_lang_types::Scalar;
use test_case::test_case;

/// The literal at the cursor `|` in `code`.
fn literal(code: &str) -> Option<String> {
    let offset = code.find('|').expect("A cursor");
    let code = code.replacen('|', "", 1);
    mucad_driver::literal_at(&code, offset).map(|(span, _)| code[span].to_string())
}

#[test_case("r = 1|0mm;", Some("10mm") ; "quantity")]
#[test_case("r = -1|0mm;", Some("-10mm") ; "negative quantity")]
#[test_case("n = |3;", Some("3") ; "integer")]
#[test_case("x = a-1|0;", Some("10") ; "subtraction")]
#[test_case("b = tr|ue;", Some("true") ; "bool")]
#[test_case("v = 1|0x;", None ; "unknown unit")]
#[test_case("s = \"1|0mm\";", None ; "in a string")]
fn literal_at(code: &str, expected: Option<&str>) {
    assert_eq!(literal(code).as_deref(), expected);
}

/// The value of the literal is returned with its span.
#[test]
fn literal_at_value() {
    let (span, value) = mucad_driver::literal_at("r = -1.5mm;", 6).expect("A literal");
    assert_eq!(span, 4..10);
    assert_eq!(
        value,
        mucad_driver::value_from_str("-1.5mm").expect("A value")
    );
}

#[test_case("10mm", 0.1, Some("10.1mm") ; "quantity by 0.1")]
#[test_case("10mm", -11.0, Some("-1mm") ; "quantity below zero")]
#[test_case("-1mm", 1.0, Some("0mm") ; "negative quantity")]
#[test_case("2.5cm", 0.1, Some("2.6cm") ; "in its unit")]
#[test_case("10.0", 1.0, Some("11.0") ; "scalar keeps decimal point")]
#[test_case("3", 10.0, Some("13") ; "integer")]
#[test_case("-7", 10.0, Some("3") ; "negative integer")]
#[test_case("3", 0.1, None ; "integer by 0.1")]
#[test_case("true", 1.0, Some("false") ; "bool")]
#[test_case("\"a\"", 1.0, None ; "string")]
#[test_case("x", 1.0, None ; "no literal")]
fn step_literal(literal: &str, step: f64, expected: Option<&str>) {
    let stepped = mucad_driver::step_literal(literal, Scalar::from_num(step));
    assert_eq!(stepped.as_deref(), expected);
}
