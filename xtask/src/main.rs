// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Parser, Subcommand};

mod update_copyright;

#[derive(Parser)]
#[command(name = "mycli")]
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
    DocTest {
        /// Filter specific tests by name
        #[arg(short, long)]
        filter: Option<String>,

        /// Run tests without executing actions
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::UpdateCopyright => {
            update_copyright::update_copyright();
        }
        Commands::DocTest { filter, dry_run } => {
            println!("Running doc tests... (dry_run = {dry_run})");
            if let Some(f) = filter {
                println!("Filter applied: {f}");
            }
            // TODO: Add doc test logic
        }
    }
}
