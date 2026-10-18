// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Display snapshot tests for IR

use mucad_lang_base::DiagRenderOptions;
mod common;

macro_rules! snapshot_test_display {
    // A successful snapshot test without errors and warnings.
    ($name:ident) => {
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
                    insta::assert_snapshot!(name, ir);
                }
                Err(errors) => panic!(
                    "{}",
                    mucad_lang_base::Diagnostics::from(errors)
                        .render_to_string(&&source, &DiagRenderOptions::default())
                        .expect("No error")
                ),
            }
        }
    };
}

snapshot_test_display!(circle);
snapshot_test_display!(builtin);
