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

/// A literal after a step, see [`step_literal_at`].
#[derive(Debug, Clone, PartialEq)]
pub struct SteppedLiteral {
    /// The literal, e.g. `1.60mm`.
    pub text: String,
    /// The byte span of the stepped digit in the text, e.g. to select it, `None` for a bool or a
    /// number with an exponent.
    pub digit: Option<mucad_lang_base::Span>,
}

/// The digits of the number in `literal`, e.g. `1.5` in `-1.5mm`, as byte span, `None` with an
/// exponent, e.g. `1e3mm`.
fn number_span(literal: &str) -> Option<mucad_lang_base::Span> {
    let start = usize::from(literal.starts_with('-'));
    let end = start
        + literal[start..]
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(literal.len() - start);
    let exponent = literal[end..].starts_with(['e', 'E'])
        && literal[end + 1..].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+');
    (start < end && !exponent).then_some(start..end)
}

/// The literal `literal` stepped by the digit at byte `cursor`, e.g. `12.4mm` for `12.3mm`, the
/// cursor in front of `3` and `factor` 1, `None` if it is no quantity, integer or bool literal.
///
/// The step is `factor` (e.g. `-10`) times the place of the digit, e.g. `0.1` for the first
/// decimal: of the digit right of the cursor, else left of it, else of the last digit, e.g. with
/// the cursor on the unit. Without digits, e.g. `1e3mm`, the step is `factor`. A bool is toggled.
///
/// The decimals are kept, e.g. `1.60mm` for `1.59mm` and `0.01`. The stepped digit is at the same
/// place, also when the literal gets wider or a sign, e.g. `10.0` for `9.9`.
pub fn step_literal_at(
    literal: &str,
    cursor: usize,
    factor: mucad_lang_types::Scalar,
) -> Option<SteppedLiteral> {
    use mucad_lang_types::Scalar;

    let value = value_from_str(literal).ok()?;
    let is_digit = |index: usize| {
        literal
            .as_bytes()
            .get(index)
            .is_some_and(u8::is_ascii_digit)
    };
    // The place of the digit, e.g. `1` for tens and `-1` for tenths, and the decimals.
    let place = number_span(literal).and_then(|number| {
        let point = literal[number.clone()]
            .find('.')
            .map_or(number.end, |i| number.start + i);
        let digit = [cursor, cursor.wrapping_sub(1)]
            .into_iter()
            .find(|index| number.contains(index) && is_digit(*index))
            .or_else(|| number.clone().rev().find(|index| is_digit(*index)))?;
        let place = match digit < point {
            true => (point - digit - 1) as i32,
            false => point as i32 - digit as i32,
        };
        let decimals = number
            .end
            .saturating_sub(point + 1)
            .max((-place).max(0) as usize);
        Some((place, decimals))
    });

    let step = match place {
        Some((place, _)) => {
            (0..place.unsigned_abs()).try_fold(factor, |step, _| match place > 0 {
                true => step.checked_mul(Scalar::from_num(10)),
                false => step.checked_div(Scalar::from_num(10)),
            })?
        }
        None => factor,
    };
    let stepped = value.step(step)?;
    let literal = stepped.literal()?;
    let text = match place {
        Some((_, decimals)) if matches!(stepped, Value::Quantity(_)) => {
            format!("{literal:.decimals$}")
        }
        _ => literal.to_string(),
    };

    // The digit at the same place in the new text.
    let digit = place.zip(number_span(&text)).map(|((place, _), number)| {
        let point = text[number.clone()]
            .find('.')
            .map_or(number.end, |i| number.start + i);
        let digit = match place >= 0 {
            true => point.saturating_sub(place as usize + 1).max(number.start),
            false => (point + place.unsigned_abs() as usize).min(number.end - 1),
        };
        digit..digit + 1
    });
    Some(SteppedLiteral { text, digit })
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
