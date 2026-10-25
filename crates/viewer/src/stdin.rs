// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Viewer Standard input interface.

use std::{io::BufRead, time::Duration};

use bevy::ecs::system::Query;
use bevy::window::Window;
use crossbeam::channel::Receiver;

use bevy::ecs::resource::Resource;
use bevy::prelude::{AppExit, MessageWriter};

use mucad_viewer_ipc::ViewerRequest;

use crate::plugin::MucadPluginInput;
use crate::processor::ProcessorRequest;
use crate::view_model::ViewerEvent;

/// A message handler for stdin messages.
#[derive(Resource, Clone)]
pub struct StdinMessageReceiver {
    receiver: Receiver<ViewerRequest>,
    current_path: Option<std::path::PathBuf>,
}

impl StdinMessageReceiver {
    /// Run the listening to standard messages.
    pub fn run() -> Self {
        log::info!("Run stdin message receiver");
        // Create channel for stdin reader to communicate with Bevy
        let (sender, receiver) = crossbeam::channel::unbounded();

        // Spawn thread to read from stdin
        std::thread::spawn(move || {
            let stdin = std::io::stdin();

            loop {
                for line in stdin.lock().lines().map_while(Result::ok) {
                    log::info!("Line: {line:?}");
                    match serde_json::from_str::<ViewerRequest>(&line) {
                        Ok(msg) => match sender.send(msg) {
                            Ok(_) => {
                                log::info!("Message sent!");
                            }
                            Err(err) => {
                                log::error!("{err}");
                            }
                        },
                        Err(e) => eprintln!("Invalid input: {e}"),
                    }
                }

                std::thread::sleep(Duration::from_millis(20));
            }
        });

        Self {
            receiver,
            current_path: None,
        }
    }

    /// The current file path.
    pub fn current_path(&self) -> &Option<std::path::PathBuf> {
        &self.current_path
    }
}

/// Process stdin messages into processor requests.
pub fn handle_stdin_messages(
    mut state: bevy::prelude::ResMut<crate::ViewModel>,
    mut exit: MessageWriter<AppExit>,
    mut windows: Query<&mut Window>,
    mut events: MessageWriter<ViewerEvent>,
) {
    let mut requests = Vec::new();
    if let Some(MucadPluginInput::Stdin(Some(stdin))) = &mut state.input {
        for viewer_request in stdin.receiver.try_iter() {
            log::info!("{viewer_request:?}");

            use mucad_viewer_ipc::ViewerRequest::*;
            match viewer_request {
                ShowSourceCodeFromFile { path } => {
                    stdin.current_path = Some(path.clone());
                    requests.push(ProcessorRequest::CompileFile(path));
                }
                ShowSourceCode { path, name, code } => {
                    stdin.current_path = path.clone();
                    requests.push(ProcessorRequest::CompileSource {
                        path,
                        name,
                        source: code,
                    });
                }
                SetCursorRange { .. } => todo!(),
                ZoomToFit => {
                    events.write(ViewerEvent::ZoomToFit);
                }
                Exit => {
                    exit.write(AppExit::Success);
                }
                Restore => match windows.single_mut() {
                    Ok(mut window) => {
                        window.set_minimized(false);
                    }
                    Err(e) => log::error!("{e}"),
                },
                Minimize => match windows.single_mut() {
                    Ok(mut window) => window.set_minimized(true),
                    Err(e) => log::error!("{e}"),
                },
            }
        }
    }

    if let Ok(mut window) = windows.single_mut() {
        state.update_window_settings(&mut window);
    }

    for request in requests {
        state.processor.send_request(request).expect("No error");
    }
}
