// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver is a high-level API to be integrated in LSP, CLI or Viewer.

mod compiler;
mod config;
mod formatter;

pub mod prelude;
//mod session;
mod watcher;

use microcad_lang_base::{DiagRenderOptions, IssueList, ToDiagnostics};
use microcad_lang_parse::ParseIssue;
use microcad_lang_types::Value;

pub use compiler::Compiler;
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
