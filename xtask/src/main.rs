// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Parser, Subcommand};

mod update_copyright;

#[derive(Parser)]
#[command(name = "microcad-xtask")]
#[command(author, version, about = "A CLI tool for copyright updates and doc testing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update copyright notices across project files
    UpdateCopyright,

    /// Run documentation tests
    DocTest,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::UpdateCopyright => update_copyright::update_copyright(),
        Commands::DocTest => Ok(()),
    }
}
