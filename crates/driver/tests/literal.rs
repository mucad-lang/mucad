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

/// The literal `literal` with the cursor `|` stepped by `factor`, with the stepped digit in
/// brackets, e.g. `1[3].5mm`.
fn step(literal: &str, factor: f64) -> Option<String> {
    let cursor = literal.find('|').expect("A cursor");
    let literal = literal.replacen('|', "", 1);
    let stepped = mucad_driver::step_literal_at(&literal, cursor, Scalar::from_num(factor))?;
    Some(match stepped.digit {
        Some(digit) => format!(
            "{}[{}]{}",
            &stepped.text[..digit.start],
            &stepped.text[digit.clone()],
            &stepped.text[digit.end..]
        ),
        None => stepped.text,
    })
}

#[test_case("|123.45mm", 1.0, Some("[2]23.45mm") ; "hundreds")]
#[test_case("1|23.45mm", 1.0, Some("1[3]3.45mm") ; "tens")]
#[test_case("12|3.45mm", 1.0, Some("12[4].45mm") ; "ones")]
#[test_case("123.|45mm", 1.0, Some("123.[5]5mm") ; "tenths")]
#[test_case("123.4|5mm", 1.0, Some("123.4[6]mm") ; "hundredths")]
#[test_case("12|.5", 1.0, Some("1[3].5") ; "digit left of the point")]
#[test_case("123.45|mm", -1.0, Some("123.4[4]mm") ; "last digit on the unit")]
#[test_case("123.45m|m", 1.0, Some("123.4[6]mm") ; "last digit inside the unit")]
#[test_case("4|5°", 1.0, Some("4[6]°") ; "degrees")]
#[test_case("1|0mm", 10.0, Some("2[0]mm") ; "shift")]
#[test_case("1.5|9", 1.0, Some("1.6[0]") ; "decimals are kept")]
#[test_case("10.0|", 1.0, Some("10.[1]") ; "scalar")]
#[test_case("9.|9", 1.0, Some("10.[0]") ; "wider")]
#[test_case("|10mm", -1.0, Some("[0]mm") ; "narrower")]
#[test_case("|0.5", -1.0, Some("-[0].5") ; "gets a sign")]
#[test_case("-|0.5", 1.0, Some("[0].5") ; "loses the sign")]
#[test_case("-1|.5mm", -1.0, Some("-[2].5mm") ; "negative")]
#[test_case("1|23", 1.0, Some("1[3]3") ; "integer")]
#[test_case("|3", -10.0, Some("-[7]") ; "negative integer")]
#[test_case("1e3|mm", 1.0, Some("1001mm") ; "exponent")]
#[test_case("tr|ue", 1.0, Some("false") ; "bool")]
#[test_case("\"a|\"", 1.0, None ; "string")]
#[test_case("x|", 1.0, None ; "no literal")]
fn step_literal_at(literal: &str, factor: f64, expected: Option<&str>) {
    assert_eq!(step(literal, factor).as_deref(), expected);
}
