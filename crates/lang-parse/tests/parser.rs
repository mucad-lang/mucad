// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Parser tests
#![cfg(feature = "parser")]

use mucad_lang_base::Diagnostics;
use test_case::test_case;

#[test_case("single int", "1")]
#[test_case("int overflow", "9999999999999999999")]
#[test_case("single float", "1.2")]
#[test_case("leading dot", ".1")]
#[test_case("trailing dot", "1.")]
#[test_case("exp", ".1e-3; .1e+3; .1e3; 1.e3; 1.2e3; 1e3")]
#[test_case("quantity int", "1mm")]
#[test_case("quantity float", ".2mm")]
#[test_case("basic addition", "1 + 1")]
#[test_case("basic addition, no space", "1+1")]
#[test_case("addition identifier", "length + 1")]
#[test_case("range", "[1..10]")]
#[test_case("list", "[1,2,3,4]")]
#[test_case("multiple binary operator", "(1+2)*3+1")]
#[test_case("assignment", "a = b * 2;")]
#[test_case("typed assignment", "a: Length = b * 2mm;")]
#[test_case("block assignment", "a = {a = 1 + 2; a * 3};")]
#[test_case("plain string", r#""plain string""#)]
#[test_case("plain string in expr", r#"a = "plain string int expression" + 1;"#)]
#[test_case("escaped bracket string", r#""string {{ with }} escaped bracket""#)]
#[test_case("escaped quote string", r#""string \" with \" escaped \ quotes""#)]
#[test_case("basic expr string", r#""string {with} expression""#)]
#[test_case("expr string", r#""string {more + complex} expression""#)]
#[test_case(
    "formatted expr string width",
    r#""string {formatted - expression:03} expression""#
)]
#[test_case(
    "formatted expr string accuracy",
    r#""string {formatted - expression:.5} expression""#
)]
#[test_case(
    "formatted expr string both",
    r#""string {formatted - expression:03.5} expression""#
)]
#[test_case("function", "fn foo(a: Length /* test */) -> Length {a * 2}")]
#[test_case("function empty args with comment", "fn foo(/* test */) {2}")]
#[test_case("function with return", "fn foo(a: Length) -> Length {return a * 2;}")]
#[test_case("function with default", "fn foo(a: Length, b = 3mm) {b / a}")]
#[test_case("call empty args", "foo()")]
#[test_case("call empty args comment", "foo(/* foobar */)")]
#[test_case(
    "call positional args",
    "foo(
        1, // foo
        2
    )"
)]
#[test_case("call named args", "foo(a = 1, b = 2)")]
#[test_case("call partially named args", "foo(a, b = 2, 3)")]
#[test_case("comment", "a = 1; // comment")]
#[test_case(
    "multiple comment",
    "// comment1
    // comment2"
)]
#[test_case(
    "multi line comment",
    r#"a = 1; /** multi
    line
    comment
    */
    b = 2;"#
)]
#[test_case(
    "doc comment",
    r#"/// Doc comment
    part Foo() {
        Cylinder(height = 10mm, radius = 5mm);
    }"#
)]
#[test_case(
    "doc comment fn",
    r#"/// Doc comment
    fn Foo() {
        1
    }"#
)]
#[test_case(
    "doc comment const",
    r#"/// Doc comment
    const A = 1;"#
)]
#[test_case(
    "doc comment property",
    r#"/// Doc comment
    prop a = 1;"#
)]
#[test_case(
    "doc comment mod",
    r#"/// Doc comment
    mod foo {
        a = 1;
    }"#
)]
#[test_case(
    "doc comment init",
    r#"/// Doc comment
    init(){}"#
)]
#[test_case(
    "inner doc comment",
    r#"//! Doc comment1
    //! Doc comment2
    //! Doc comment3
    "#
)]
#[test_case("tuple", "(1, 1 + 1)")]
#[test_case("one-tuple", "(1,)")]
#[test_case("one-bracketed", "(1)")]
#[test_case("named tuple", "(length = 1, width = 1 + 1)")]
#[test_case("named one-tuple, trailing", "(length = 1,)")]
#[test_case("named one-tuple", "(length = 1)")]
#[test_case("partially named tuple", "(\"a\", length = 1)")]
#[test_case("empty tuple", "()")]
#[test_case("symbol path", "foo::bar")]
#[test_case("symbol path prefix", "::foo::bar")]
#[test_case("marker", "@input")]
#[test_case("unary", "!input")]
#[test_case("if", "if a > 1 { foo(); }")]
#[test_case("if-else", "if a > 1 { 3 } else { 4 }")]
#[test_case("if-bracket", "if (a > 1) { foo(); }")]
#[test_case("else-if", "if a > 1 { 3 } else if a < -1 { 1 }")]
#[test_case("else-if-else", "if a > 1 { 3 } else if a < -1 { 1 } else { 0 }")]
#[test_case("sketch", "sketch Wheel(radius: Length) {std::geo2d::Circle(radius);}")]
#[test_case(
    "pub-part",
    "pub part Wheel(radius: Length, height = 1mm) {std::geo3d::Cylinder(radius, height);}"
)]
#[test_case(
    "sketch-with-init",
    "sketch Wheel(radius: Length) {
    init(diameter: Length) {
        radius = diameter / 2;
    }
    std::geo2d::Circle(radius);
}"
)]
#[test_case("mod", "mod foo { fn bar(){} }")]
#[test_case("mod pub", "pub mod foo { fn bar(){} }")]
#[test_case("mod extern", "mod foo;")]
#[test_case("use", "use foo;")]
#[test_case("use glob", "use foo::bar::*;")]
#[test_case("use prefix", "use ::foo::bar;")]
#[test_case("use as", "pub use foo::bar as foobar;")]
#[test_case("use super", "use super::super::foo::*;")]
#[test_case("super path", "super::foo::bar()")]
#[test_case("super as identifier", "super = 1;")]
#[test_case("list units", "a = [1, 2]mm;")]
#[test_case("range units", "a = [1..3]mm;")]
#[test_case("const", "const FOO = 1;")]
#[test_case("pub const assignment", "pub const FOO = 1;")]
#[test_case("pub assignment", "pub FOO = 1;")]
#[test_case(
    "attribute assignment",
    r##"#[color = "#FF00FF"]
    a = 1;"##
)]
#[test_case(
    "attribute expression",
    r##"#[color = "#FF00FF"]
    1;"##
)]
#[test_case(
    "attribute trailing expression",
    r##"#[color = "#FF00FF"]
    1"##
)]
#[test_case(
    "attribute workbench",
    r##"#[color = "#FF00FF"]
    part Foo() {
        Cylinder(height = 10mm, radius = 5mm);
    }"##
)]
#[test_case(
    "attribute call",
    r##"#[svg(style = "fill:none")]
    a = 1;"##
)]
#[test_case(
    "attribute ident",
    r##"#[export]
    a = 1;"##
)]
#[test_case("attribute access", r##"foo#bar"##)]
#[test_case("list access", r##"foo[1]"##)]
#[test_case("tuple access", r##"foo.bar"##)]
#[test_case("method call", r##"foo.bar()"##)]
#[test_case("multiple element access", r##"foo[1]#bar.asd.call(1)"##)]
#[test_case("percent unit", "1%")]
#[test_case(
    "unclosed string",
    r#"a = "unclosed string;
    b = 1;"#
)]
#[test_case("units", r#"0" + 1% + 2mm / 3mm2 - 4mm³ * 1° + 1" - 2'"#; "units")]
#[test_case("list binary ops", r#"[1,2]" == [1,2]cm"#)]
#[test_case("unary binary ops", r#"-[1] != [1]"#)]
#[test_case("list comments", r#"[1 /* foo */,2 /* bar */]"#)]
#[test_case("un-typeable", r#"b = "sad"m3;"#)]
#[test_case("binary op precedence", "a^2 + b^3")]
#[test_case("inner attribute", r##"#![export]"##)]
#[test_case(
    "single trailing expr",
    r#"pub part A() {
        {
            B(C);
        }.align(D);
    }"#
)]
#[test_case("body expr method tail", "{}.foo()")]
#[test_case("body expr method", "{}.foo();")]
#[test_case("body expr and_tail_expr", "{}bar.foo();")]
fn test_parser(name: &str, input: &str) -> miette::Result<()> {
    use miette::IntoDiagnostic;

    use mucad_lang_base::{Artifact, DiagRenderOptions, Source};

    let source = Source::from(input);
    let result = mucad_lang_parse::parse(&source);

    // 1. Extract AST (or empty if failed)
    let ast_str = result.as_ref().map_or(String::default(), |(ast, _)| {
        ast.to_ron()
            .unwrap_or_else(|e| format!("Serialization error: {e}"))
    });

    // 2. Extract Diagnostics
    let errors = result.map_or_else(|errors| errors, |(_, errors)| errors);
    let diagnostics = Diagnostics::from(errors);
    let diag_str = diagnostics
        .render_to_string(
            &&source,
            &DiagRenderOptions {
                color: false,
                unicode: false,
            },
        )
        .into_diagnostic()?;

    let output = format!("{ast_str}\n---\n{diag_str}");
    Ok(insta::assert_snapshot!(format!("parser_{name}"), output))
}
