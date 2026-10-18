// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Viewer IPC interface

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The cursor position to be sent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorPosition {
    line: usize,
    col: usize,
}

/// A request sent to the viewers stdin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ViewerRequest {
    /// Show source code from file.
    ShowSourceCodeFromFile {
        /// File path.
        path: PathBuf,
    },
    /// Show a source code snippet.
    ShowSourceCode {
        /// An optional path to the file, if this snippet is part of a file.
        path: Option<PathBuf>,
        /// An optional name for the source code snippet.
        name: Option<String>,
        /// The actual source code.
        code: String,
    },
    /// Set the current cursor range.
    SetCursorRange {
        /// Begin of the cursor range.
        begin: Option<CursorPosition>,
        /// End of the cursor range.
        end: Option<CursorPosition>,
    },
    /// Hide window.
    Restore,
    /// Hide window.
    Minimize,
    /// Set zoom level to 100%, so we can see the entire model.
    ZoomToFit,
    /// Exit viewer process.
    Exit,
}

/// A response sent from the viewers stdout
#[derive(Clone, Serialize, Deserialize)]
pub enum ViewerResponse {
    /// The viewer sent a status message.
    StatusMessage(String),
}

/// Our interface to the viewer process.
#[derive(Debug)]
pub struct ViewerProcessInterface {
    request_sender: crossbeam::channel::Sender<ViewerRequest>,
    _response_receiver: crossbeam::channel::Receiver<ViewerResponse>,
}

impl ViewerProcessInterface {
    /// Send a request to the viewer process.
    pub fn send_request(&self, request: ViewerRequest) -> miette::Result<()> {
        self.request_sender.send(request).into_diagnostic()
    }

    /// Run the viewer process.
    pub fn run(show_window: bool) -> Self {
        log::info!("Running viewer Process");

        use crossbeam::channel::*;

        let (tx, rx): (Sender<ViewerRequest>, Receiver<ViewerRequest>) = unbounded();
        let (_, resp_rx) = unbounded::<ViewerResponse>();

        std::thread::spawn(move || {
            // Spawn slave process
            let mut command = std::process::Command::new(
                std::env::var("MICROCAD_VIEWER_BIN").unwrap_or("mucad-viewer".to_string()),
            );
            // handle multiple search paths
            if !show_window {
                command.arg("--hidden");
            }
            let mut child = command
                .arg("--stay-on-top")
                .arg("-v")
                .arg("stdin://") // run the viewer as slave via stdin.
                .current_dir(std::env::current_dir().expect("current dir"))
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("Failed to start slave process");

            let mut stdin = child.stdin.take().expect("stdin");
            let stdout = child.stdout.take().expect("stdout");

            // Thread to read responses
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    log::info!("Response: {line}");
                    // TODO: Read viewer responses.
                    // if let Ok(resp) = serde_json::from_str::<ViewerResponse>(&line) {
                    //     resp_tx.send(resp).unwrap();
                    // }
                }
            });

            // Thread to write requests
            std::thread::spawn(move || {
                loop {
                    for req in &rx {
                        use std::io::Write;
                        match serde_json::to_string(&req) {
                            Ok(json) => {
                                log::debug!("Write request as json: {json}");
                                writeln!(stdin, "{}", json).expect("io error");
                                stdin.flush().expect("io error");
                            }
                            Err(_) => todo!(),
                        };
                    }
                }
            });

            child.wait().expect("No timeout");
        });

        Self {
            request_sender: tx,
            _response_receiver: resp_rx,
        }
    }
}
