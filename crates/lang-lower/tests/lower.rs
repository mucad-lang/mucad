// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Artifact, DiagRenderOptions};
use mucad_lang_lower::{self as lower, Ir};
use mucad_lang_parse::{
    self as parse, Ast, Parse,
    ast::visitor::{ExpectedDiagnostic, ExpectedDiagnostics},
};

mod common;

use parse::ast;

use test_that::prelude::*;

macro_rules! _unit_test {
    ($name:ident => |$ir:ident, $diag:ident| $body:block) => {
        #[test_that::test]
        fn $name() {
            let source = source_from_test_file(stringify!($name));
            match ir_from_source(&source) {
                Ok(($ir, $diag)) => $body,
                Err(err) => panic!(
                    "{}",
                    err.render_to_string(&&source, &DiagRenderOptions::default())
                        .expect("No error")
                ),
            }
        }
    };
}
macro_rules! snapshot_test {
    // A successful snapshot test without errors and warnings.
    ($name:ident => ok) => {
        #[test_that::test]
        fn $name() {
            let name = stringify!($name);
            let source = common::source_from_test_file(name);
            match common::ir_from_source(&source) {
                Ok((ir, errors)) => {
                    let diags = mucad_lang_base::Diagnostics::from(errors);
                    if diags.has_errors() || diags.has_warnings() {
                        panic!("{diags:?}");
                    }
                    insta::assert_snapshot!(name, ir.to_ron().expect("No error"));
                }
                Err(errors) => {
                    let diags = mucad_lang_base::Diagnostics::from(errors);
                    panic!(
                        "{}",
                        diags
                            .render_to_string(&&source, &DiagRenderOptions::default())
                            .expect("No error")
                    )
                }
            }
        }
    };

    // A successful snapshot test with (expected) errors.
    ($name:ident => error) => {
        #[test]
        fn $name() {
            let name = stringify!($name);
            let source = source_from_test_file(name);
            match ir_from_source(&source) {
                Ok((ir, diag)) => {
                    assert!(diag.has_errors());
                    insta::assert_snapshot!(name, ir.to_ron().expect("No error"));
                }
                Err(err) => panic!(
                    "{}",
                    err.render_to_string(&&source, &DiagRenderOptions::default())
                        .expect("No error")
                ),
            }
        }
    };
}

macro_rules! test_diagnostic {
    ($name:ident) => {
        #[test]
        fn $name() {
            let filename = stringify!($name);
            let source = common::source_from_test_file(filename);
            let parse_context = mucad_lang_parse::ParseContext::from(&source);
            let ast = Ast::parse(&parse_context).unwrap();
            let mut context = mucad_lang_lower::LowerContext::new(&source);
            let expected = ast::visitor::collect_expected_diagnostics(&parse_context, &ast);
            let no_diagnostics_expected = expected.is_empty();

            // Errors may be recovered from (`Ok` with issues) or not (`Err`).
            let (ir, issues) = match lower::lower(&mut context, &ast) {
                Ok((ir, issues)) => (Some(ir), issues),
                Err(issues) => (None, issues),
            };

            let diags = mucad_lang_base::Diagnostics::from(issues);
            let observed = ExpectedDiagnostics::new(
                diags
                    .iter()
                    .map(|diag| ExpectedDiagnostic {
                        severity: diag.severity().unwrap_or_default(),
                        line: diag.src_ref.line().unwrap(),
                        code: diag.code().map(|code| code.to_string()),
                    })
                    .collect(),
            );

            // Display diagnostics on failure
            assert_that!(
                observed,
                eq(expected),
                "{}",
                diags
                    .render_to_string(&&source, &Default::default())
                    .unwrap(),
            );

            // Snapshot the IR of tests without expected diagnostics.
            if no_diagnostics_expected {
                let mut ir = ir.unwrap_or_else(|| panic!("Test '{filename}' produced no IR"));
                use mucad_lang_lower::ir::visitor::VisitorMut;
                mucad_lang_lower::ir::visitor::MakeHumanReadable::new(&context).visit(&mut ir);

                insta::assert_snapshot!(stringify!($name), ir);
            }
        }
    };
}

snapshot_test!(circle => ok);
snapshot_test!(builtin => ok);

/// An unknown built-in is reported, but the rest of the source is still lowered.
#[test]
fn unknown_builtin() {
    let source = mucad_lang_base::Source::from(
        "__mu::geo2d::Unknown(radius = 1mm);\n__mu::geo2d::Circle(radius = 2mm);\n",
    );
    let (ir, issues) = common::ir_from_source(&source).expect("An IR despite the error");

    let errors: Vec<_> = issues.iter().map(|issue| issue.to_string()).collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("No built-in `__mu::geo2d::Unknown`"),
        "{errors:?}"
    );

    let ron = ir.to_ron().expect("No error");
    assert!(ron.contains("Unknown"), "{ron}");
    assert!(ron.contains("Circle"), "{ron}");
}

test_diagnostic!(unexpected_statements);
test_diagnostic!(init);
test_diagnostic!(init_statement);
test_diagnostic!(rect);

/// Test serialization
#[test_that::test]
fn serde_circle() {
    let source = common::source_from_test_file("circle");
    let in_ir = common::ir_from_source(&source).unwrap().0;

    // 1. Serialize/Deserialize
    let serialized = in_ir.to_ron().expect("No error");
    let out_ir = Ir::from_ron(&*serialized).expect("Deserialization failed");

    // 2. Structural Equality
    assert_that!(
        &in_ir,
        eq(&out_ir),
        "IR structure changed after round-trip!"
    );

    // 3. String Identity
    let re_serialized = out_ir.to_ron().expect("Re-serialization failed");
    assert_that!(
        serialized,
        eq(re_serialized),
        "RON string output changed after round-trip!"
    );
}
