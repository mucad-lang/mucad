// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad command line interpreter

extern crate clap;

mod cli;
mod commands;
mod share;
pub use cli::*;
use commands::*;

/// Main of the command line interpreter
fn main() -> miette::Result<()> {
    let cli = Cli::new()?;

    // Initialize env_logger with a default filter level
    env_logger::Builder::from_default_env()
        .filter_level(match cli.verbose {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            3 => log::LevelFilter::Trace,
            _ => panic!("unknown verbosity level"),
        }) // Set the default log level
        .init();

    cli.run()
}
