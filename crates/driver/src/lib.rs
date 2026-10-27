// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver is a high-level API to be integrated in LSP, CLI or Viewer.

mod compiler;
mod config;
mod exporter;
mod formatter;

pub mod prelude;
pub mod share;

#[cfg(not(target_arch = "wasm32"))]
mod watcher;

use mucad_lang_base::{DiagRenderOptions, IssueList, ToDiagnostics};
use mucad_lang_parse::ParseIssue;
use mucad_lang_types::Value;

pub use compiler::Compiler;
pub use exporter::{ExportFile, Exporter};
pub use formatter::Formatter;

/// We use [`miette::Result`] throught-out this crate.
pub type Result<T = ()> = miette::Result<T>;

/// Expose miette report.
pub use miette::Report;

/// Wrapper for miette macro
pub fn report(s: &str) -> Report {
    miette::miette!("{s}")
}

pub use config::DriverConfig;

/// Parse a render resolution from a string, e.g. a length (`0.05mm`) or a percentage (`200%`).
///
/// A percentage is relative to the default resolution: `200%` is twice as fine.
pub fn resolution_from_str(s: &str) -> Result<prelude::RenderResolution> {
    use mu::RenderResolution;
    use mu::types::{QuantityType, Unit, math::IntoFloat};
    use prelude as mu;

    let invalid = || {
        miette::miette!(
            "Invalid resolution `{s}`, expected a length (e.g. `0.05mm`) or a percentage (e.g. `200%`)"
        )
    };

    let Value::Quantity(q) = value_from_str(s)? else {
        return Err(invalid());
    };
    let value: f64 = q.value.into_float();
    match (q.quantity_type, q.unit) {
        _ if value <= 0.0 => Err(invalid()),
        (QuantityType::Length, _) => Ok(RenderResolution::new(value)),
        (_, Unit::Percent) => Ok(RenderResolution::new(
            RenderResolution::default().linear / value,
        )),
        _ => Err(invalid()),
    }
}

/// Parse a value from a string containing a literal, e.g. `10mm`, `3` or `true`.
///
/// A leading `-` negates a quantity or an integer, e.g. `-1.5mm` or `-3`, though the sign is no
/// part of a literal in code, but a negation. Whitespace around the literal is ignored.
pub fn value_from_str(s: &str) -> Result<Value> {
    use mucad_lang_types::Quantity;

    let s = s.trim();
    // A number follows the sign, e.g. not `- 3` or `--3`.
    let positive = (s.strip_prefix('-'))
        .filter(|rest| rest.starts_with(|c: char| c.is_ascii_digit() || c == '.'));
    match positive {
        Some(positive) => match literal_from_str(positive)? {
            Value::Quantity(quantity) => Ok(Value::Quantity(Quantity {
                value: -quantity.value,
                ..quantity
            })),
            Value::Integer(integer) => Ok(Value::Integer(-integer)),
            value => Err(miette::miette!("The literal `{value}` cannot be negated")),
        },
        None => literal_from_str(s),
    }
}

/// The span and the value of the literal at byte `offset` in `code`, also with `offset` right
/// after it, e.g. `10mm` in `r = 10mm;`, to change it in an editor, see
/// [`mucad_lang_parse::literal_at`].
///
/// Only quantities, integers and bools are literals, e.g. not `10x` with an unknown unit.
pub fn literal_at(code: &str, offset: usize) -> Option<(mucad_lang_base::Span, Value)> {
    let span = mucad_lang_parse::literal_at(code, offset)?;
    let value = value_from_str(&code[span.clone()]).ok()?;
    value.literal().is_some().then_some((span, value))
}

/// The literal `literal` increased by `step` in its unit, e.g. `10.1mm` for `10mm` and `0.1`,
/// see [`Value::step`], `None` if it is no quantity, integer or bool literal or does not change.
pub fn step_literal(literal: &str, step: mucad_lang_types::Scalar) -> Option<String> {
    let value = value_from_str(literal).ok()?.step(step)?;
    Some(value.literal()?.to_string())
}

/// Parse a value from a string containing a literal without a sign, see [`value_from_str`].
fn literal_from_str(s: &str) -> Result<Value> {
    use mu::lower::Desugar;
    use mu::parse::Parse;
    use prelude as mu;

    let source = mu::Source::from(s);
    let parse_context = mu::parse::ParseContext::from(&source);
    let node = mu::ast::Literal::parse(&parse_context).map_err(|errors| {
        let issues: IssueList<ParseIssue> = IssueList::from(errors);
        miette::miette!(
            issues
                .to_diagnostics()
                .render_to_string(&source, &DiagRenderOptions::default())
                .unwrap()
        )
    })?;

    mu::ir::ConstantValue::desugar(&node, &mut mu::lower::LowerContext::from(&source))
        .map_err(|err| err.into())
        .map(|lit| lit.value().clone())
}
