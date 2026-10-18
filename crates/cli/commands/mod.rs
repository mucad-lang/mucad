// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad CLI commands

mod check;
mod compile;
mod completions;
mod create;
mod doc;
mod export;
mod format;
mod watch;

use clap::Subcommand;

pub use check::Check;
pub use compile::Compile;
pub use create::Create;
pub use doc::Doc;
pub use export::Export;
pub use format::Format;
pub use watch::Watch;

use crate::commands::completions::Completions;

#[derive(Subcommand)]
pub enum Commands {
    /// Check a µcad file.
    Check(Check),

    /// Compile a µcad for debugging.
    Compile(Compile),

    /// Compile and export a µcad file.
    Export(Export),

    /// Create a new source file with µcad extension.
    Create(Create),

    /// Watch a µcad file
    Watch(Watch),

    /// Format a µcad file
    #[clap(alias = "fmt")]
    Format(Format),

    /// Generate Markdown docs.
    Doc(Doc),

    /// Print shell completions
    Completions(Completions),
}

/// Run this command for a CLI.
pub trait RunCommand<T = ()> {
    fn run(&self, cli: &crate::cli::Cli) -> miette::Result<T>;
}
