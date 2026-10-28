// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::Source;

fn _formatted_test_case(name: &str) {
    use mucad_lang_format::{FormatConfig, format_ast};
    let source = Source::load(format!("tests/test_cases/formatted/{name}.mu")).expect("No errors");
    let ast = mucad_lang_parse::parse(&source).expect("No errors").0;

    pretty_assertions::assert_eq!(
        source.code(),
        format_ast(&ast, &FormatConfig::default()),
        "Format error:\n{ast:#?}",
    );
}

fn _unformatted_test_case(name: &str) {
    use mucad_lang_format::{FormatConfig, format_ast};
    let source =
        Source::load(format!("tests/test_cases/unformatted/{name}.mu")).expect("no errors");
    let ast = mucad_lang_parse::parse(&source).expect("No errors").0;
    insta::assert_snapshot!(name, format_ast(&ast, &FormatConfig::default()))
}

// A test case whose format output does not change.
macro_rules! formatted_test_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            _formatted_test_case(stringify!($name));
        }
    };
}

// A test case whose format output does not change.
macro_rules! unformatted_test_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            _unformatted_test_case(stringify!($name));
        }
    };
}

formatted_test_case!(statement_list_whitespace);
formatted_test_case!(list);
formatted_test_case!(tuple);
formatted_test_case!(tuple_type);
formatted_test_case!(body);
formatted_test_case!(expression);
formatted_test_case!(expression_if);
formatted_test_case!(statements);
formatted_test_case!(workbench);
formatted_test_case!(extras);
formatted_test_case!(extras_module);
formatted_test_case!(extras_multiline_comment);
formatted_test_case!(extras_workbench);
formatted_test_case!(definition);
formatted_test_case!(string);

unformatted_test_case!(init);
