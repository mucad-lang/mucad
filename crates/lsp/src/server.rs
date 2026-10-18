// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language server.

use mucad_driver::prelude as mu;
use mucad_lsp as mu_lsp;

use clap::Parser;
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// log into given file
    #[arg(short, long, value_name = "FILE")]
    log_file: Option<std::path::PathBuf>,

    #[arg(long)]
    stdio: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Some(log_file) = args.log_file {
        let file = std::fs::File::create(log_file).expect("could not create log file");
        let target = Box::new(file);
        env_logger::Builder::new()
            .target(env_logger::Target::Pipe(target))
            .filter(None, log::LevelFilter::Info)
            .init();
    } else {
        env_logger::try_init_from_env("MICROCAD_LSP_LOG").ok();
    }

    let config = mu_lsp::Config::default();

    let (service, socket) = mu_lsp::build_lsp_service(config).expect("No error");
    log::info!("LSP service has been created");

    if args.stdio {
        use tokio::io::{stdin, stdout};
        mu_lsp::Server::new(stdin(), stdout(), socket)
            .serve(service)
            .await;
    } else {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:5007")
            .await
            .expect("bind listener to 127.0.0.1:5007");
        log::info!("LSP server listening...");
        let (stream, _) = listener.accept().await.expect("accept socket");
        log::info!("Client has connected to LSP service");
        let (read, write) = tokio::io::split(stream);
        mu_lsp::Server::new(read, write, socket)
            .serve(service)
            .await;
    };
}
