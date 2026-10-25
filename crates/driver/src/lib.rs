// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver is a high-level API to be integrated in LSP, CLI or Viewer.

mod compiler;
mod config;
mod exporter;
mod formatter;

pub mod prelude;
pub mod share;
//mod session;
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

/// Parse a value from a string containing a literal.
pub fn value_from_str(s: &str) -> Result<Value> {
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
