// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for visitor pattern.

use insta::assert_debug_snapshot;
use miette::Severity;
use mucad_lang_parse::{
    Ast, Parse, ParseContext,
    ast::visitor::{CommentCollector, ExpectedDiagnostic, ExpectedDiagnosticsCollector, Visit},
};
use test_case::test_case;

#[test_case(
    "comment collector",
    "
    // A
    foo(
        1, // B
        2
    );
    // C
    "
)]
fn comment_collector(name: &str, input: &str) {
    let ast = mucad_lang_parse::parse(&mucad_lang_base::Source::from(input))
        .expect("No error")
        .0;
    let mut comment_collector = CommentCollector::default();
    let _ = ast.tree().visit(&mut comment_collector);

    assert_debug_snapshot!(format!("visitor_{name}"), comment_collector);
}

#[test]
fn expected_diagnostic_from_comment() {
    let line = 10;
    let diag1 = ExpectedDiagnostic::from_comment("// error: UnexpectedToken", line).expect("Some");

    assert_eq!(diag1.severity, Severity::Error);
    assert_eq!(diag1.code.unwrap(), "UnexpectedToken");

    let diag2 = ExpectedDiagnostic::from_comment("// warning", line).expect("Some");
    assert_eq!(diag2.severity, Severity::Warning);
    assert!(diag2.code.is_none());

    assert!(ExpectedDiagnostic::from_comment("// Regular comment", line).is_none());
}

#[test_case(
    "expected diagnostic collector",
    "
    foo(1, 2); // error: SomeErrorCode
    // warning: SomeCode
    "
)]
fn expected_diagnostics_collector(name: &str, input: &str) {
    let source = mucad_lang_base::Source::from(input);
    let context = ParseContext::from(&source);
    let ast = Ast::parse(&context).expect("No parse errors");

    let mut collector = ExpectedDiagnosticsCollector::new(&context);
    let _ = ast.tree().visit(&mut collector);

    let diags = collector.diagnostics();

    assert_debug_snapshot!(format!("visitor_{name}"), diags);
}

/// The single identifier of an expression is the only operand of operators with an identifier,
/// which appears only once in the expression.
#[test_case("r" => Some("r".into()); "identifier")]
#[test_case("-r" => Some("r".into()); "unary operation")]
#[test_case("r * 2%" => Some("r".into()); "binary operation")]
#[test_case("std::math::PI * r" => Some("r".into()); "path")]
#[test_case("r * sin(a)" => Some("r".into()); "call")]
#[test_case("r + w" => None; "two identifiers")]
#[test_case("[1..n] / n * 360°" => None; "identifier twice")]
#[test_case("r * sin(r)" => None; "identifier twice in call")]
#[test_case("[1..n] / 2" => None; "range")]
#[test_case("m.radius" => None; "element access")]
#[test_case("sqrt(r)" => None; "only call")]
#[test_case("42mm" => None; "literal")]
fn single_identifier(expr: &str) -> Option<String> {
    let ast = mucad_lang_parse::parse(&mucad_lang_base::Source::from(expr))
        .expect("No error")
        .0;
    let tail = ast.tree().statements.tail.as_ref().expect("An expression");
    tail.expr.single_identifier().map(|name| name.to_string())
}
