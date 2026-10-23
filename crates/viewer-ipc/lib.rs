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

/// A viewer, which can be controlled with requests, e.g. a [`ViewerProcess`].
pub trait Viewer: std::fmt::Debug + Send + Sync {
    /// Send a request to the viewer.
    fn send_request(&self, request: ViewerRequest) -> miette::Result<()>;
}

/// The command to start a viewer process, which reads [`ViewerRequest`]s as JSON lines from stdin.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewerCommand {
    /// The program, e.g. `mucad-viewer`.
    pub program: String,
    /// The arguments of the program.
    pub args: Vec<String>,
}

impl ViewerCommand {
    /// A command to start `program` without arguments.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// The µcad viewer, `mucad-viewer` or the program in the environment variable
    /// `MICROCAD_VIEWER_BIN`, staying on top of other windows and reading requests from stdin.
    pub fn mucad_viewer(show_window: bool) -> Self {
        let program =
            std::env::var("MICROCAD_VIEWER_BIN").unwrap_or_else(|_| "mucad-viewer".to_string());
        let command = Self::new(program);
        let command = if show_window {
            command
        } else {
            command.arg("--hidden")
        };
        command.arg("--stay-on-top").arg("-v").arg("stdin://")
    }
}

/// A viewer in a separate process, which gets the requests as JSON lines on its stdin.
#[derive(Debug)]
pub struct ViewerProcess {
    request_sender: crossbeam::channel::Sender<ViewerRequest>,
    _response_receiver: crossbeam::channel::Receiver<ViewerResponse>,
}

impl Viewer for ViewerProcess {
    fn send_request(&self, request: ViewerRequest) -> miette::Result<()> {
        self.request_sender.send(request).into_diagnostic()
    }
}

impl ViewerProcess {
    /// Start the viewer process of `command`.
    pub fn run(command: ViewerCommand) -> Self {
        log::info!("Running viewer process {command:?}");

        use crossbeam::channel::*;

        let (tx, rx): (Sender<ViewerRequest>, Receiver<ViewerRequest>) = unbounded();
        let (_, resp_rx) = unbounded::<ViewerResponse>();

        std::thread::spawn(move || {
            // Spawn slave process
            let mut child = std::process::Command::new(&command.program)
                .args(&command.args)
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
