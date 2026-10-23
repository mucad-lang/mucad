// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{IssueResult, MUCAD_EXTENSION, Source};
use mucad_lang_lower::{self as lower, Ir, LowerContext, LowerIssue, ir};
use mucad_lang_parse as parse;

/// Get intermediate representation and diagnostics.
#[allow(unused)]
pub fn ir_from_source(source: &Source) -> IssueResult<Ir, LowerIssue> {
    let ast = parse::parse(source).expect("No parse errors").0;
    let mut context = LowerContext::from(source);
    let (mut ir, issues) = lower::lower(&ast, &mut context)?;

    use mucad_lang_lower::ir::visitor::VisitorMut;
    ir::visitor::MakeHumanReadable::new(&context).visit(&mut ir);

    Ok((ir, issues))
}

#[allow(unused)]
pub fn source_from_test_file(name: &str) -> Source {
    Source::load(format!("tests/test_cases/{name}.{}", MUCAD_EXTENSION)).expect("No error")
}
