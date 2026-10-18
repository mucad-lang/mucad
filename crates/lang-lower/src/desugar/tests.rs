// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for desugaring

use crate::{LowerContext, LowerResult, ir};
use mucad_lang_base::Source;
use mucad_lang_parse::parse;

mod common {
    use mucad_lang_base::MUCAD_EXTENSION;

    use crate::Desugar;

    use super::*;

    pub fn desugar(source: &Source) -> LowerResult<ir::desugared::Source> {
        let ast = parse(source).expect("No parse error").0;
        let mut context = LowerContext::from(source);
        ir::desugared::Source::desugar(ast.tree(), &mut context)
    }

    pub fn source_from_test_file(name: &str) -> Source {
        Source::load(format!("tests/test_cases/{name}.{}", MUCAD_EXTENSION)).expect("No error")
    }
}

macro_rules! desugar_unit_test {
    ($name:ident => |$ir:ident| $body:block) => {
        #[test_that::test]
        fn $name() {
            let source = common::source_from_test_file(stringify!($name));
            common::desugar(&source).expect("No error")
        }
    };
}

desugar_unit_test!(module => |ir| {
    assert_that!(ir.tree, matches_pattern!(ir::Source {
        *statements: len(eq(3)),
        items: matches_pattern!(ir::SourceItems {
            constants: len(eq(1)),
            inline_modules: [matches_pattern!(ir::InlineModule {
                visibility: eq(ir::Visibility::Private),
                items: matches_pattern!(ir::InlineModuleItems {
                    constants: len(eq(1)),
                    functions: [matches_pattern!(ir::Function {
                        visibility: eq(ir::Visibility::Public),
                    })],
                    workbenches: [matches_pattern!(ir::Workbench {
                        id: eq(Identifier::from("MySketch")),
                        visibility: eq(ir::Visibility::Public),
                    })]
                })
            })]
        })
    }));

});

desugar_unit_test!(inline_module => |ir| {
    assert_that!(ir.tree, matches_pattern!(ir::Source {
        statements: empty(),
        items: matches_pattern!(ir::SourceItems {
            constants: len(eq(1)),
            inline_modules: [matches_pattern!(ir::InlineModule {
                visibility: eq(ir::Visibility::Public),
                id: eq(Identifier::from("b")),
                items: matches_pattern!(ir::InlineModuleItems {
                    constants: [matches_pattern!(
                        ir::Constant {
                            id: eq(ir::Identifier::from("C")),
                            visibility: eq(ir::Visibility::Public),
                        }
                    )],
                    modules: [
                        matches_pattern!(ir::InlineModule {
                            id: eq(Identifier::from("d")),
                            visibility: eq(ir::Visibility::Public),
                            items: matches_pattern!(ir::InlineModuleItems {
                                constants: [matches_pattern!(
                                    ir::Constant {
                                        id: eq(Identifier::from("E")),
                                        visibility: eq(ir::Visibility::Private),
                                    }
                                )],
                            }),
                        }),
                        matches_pattern!(ir::InlineModule {
                            id: eq(Identifier::from("f")),
                            items: matches_pattern!(ir::InlineModuleItems {
                                constants: [matches_pattern!(
                                    ir::Constant {
                                        id: eq(Identifier::from("G")),
                                        visibility: eq(ir::Visibility::Private),
                                    }
                                )],
                            }),

                        })
                    ],
                })
            })]
        })
    }));

    assert!(!diag.has_errors())
});
