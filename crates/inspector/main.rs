// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad inspector

#![allow(missing_docs)]

use clap::Parser;
use mucad_driver::prelude as mu;

use crossbeam::channel::Sender;
use mucad_viewer_ipc::{ViewerProcessInterface, ViewerRequest};
use miette::IntoDiagnostic;
use std::sync::{Arc, RwLock};
use std::thread;

use slint::VecModel;

use crate::to_slint::hash_to_shared_string;

slint::include_modules!();

mod symbol_info;
mod to_slint;

#[derive(Parser)]
struct Args {
    /// Input µcad file.
    pub input: String,

    /// Paths to search for files.
    #[arg(short = 'P', long = "search-path", action = clap::ArgAction::Append, default_value = "./crates/std/lib", global = true)]
    pub search_paths: Vec<std::path::PathBuf>,
}

/// A request to the view model.
#[derive(Debug)]
pub enum ViewModelRequest {
    /// Set source code string.
    SetSourceCode {
        /// Source code to be displayed.
        code: mu::Hashed<String>,
    },
    /// Set the symbol tree items.
    SetSymbolTree(Vec<SymbolTreeModelItem>),
    /// Set the model tree items.
    SetModelTree(Vec<ModelTreeModelItem>),
    /// The current line of source code.
    SetCurrentLine(u64),
}

struct Inspector {
    args: Args,

    pub watcher: mu::Watcher,

    pub viewer_process: Arc<RwLock<Option<mucad_viewer_ipc::ViewerProcessInterface>>>,
}

impl Inspector {
    pub fn new() -> miette::Result<Self> {
        Ok(Self {
            args: Args::parse(),
            watcher: mu::Watcher::new()?,
            viewer_process: Arc::new(RwLock::new(None)),
        })
    }

    pub fn run(mut self) -> miette::Result<()> {
        // Create the Slint UI component
        let main_window = MainWindow::new().into_diagnostic()?;

        let weak = main_window.as_weak();
        let input = self.args.input.clone();
        let (tx, rx): (Sender<ViewModelRequest>, _) = crossbeam::channel::unbounded();
        let search_paths = self.args.search_paths.clone();

        // Run file watcher thread.
        std::thread::spawn(move || -> miette::Result<()> {
            loop {
                use mu::traits::*;

                use crate::to_slint::ItemsFromTree;
                // Watch all dependencies of the most recent compilation.
                self.watcher.update(vec![self.args.input.clone().into()])?;

                let search_paths = search_paths.clone();

                let mut document = mu::document::SourceFile::load_from_file(
                    mu::locate::to_url(&self.args.input)?,
                    0,
                )?;
                tx.send(ViewModelRequest::SetSourceCode {
                    code: mu::Hashed::new(
                        document.get_code().map(|code| code.to_string()).unwrap(),
                    ),
                })
                .into_diagnostic()?;

                document
                    .parse()
                    .and(document.lower())
                    .and(document.resolve(mu::ResolveParameters {
                        search_paths,
                        no_builtin: false,
                    }))
                    .and_then(|symbol| {
                        tx.send(ViewModelRequest::SetSymbolTree({
                            symbol
                                .iter()
                                .flat_map(|s| SymbolTreeModelItem::items_from_tree(&s))
                                .collect()
                        }))
                        .into_diagnostic()
                    })?;

                document.eval().and_then(|model| {
                    tx.send(ViewModelRequest::SetModelTree(
                        ModelTreeModelItem::items_from_tree(&model),
                    ))
                    .into_diagnostic()
                })?;

                // Wait until anything relevant happens.
                self.watcher.wait()?;
            }
        });

        thread::spawn(move || {
            loop {
                if let Ok(request) = rx.recv() {
                    weak.upgrade_in_event_loop(move |main_window| match request {
                        ViewModelRequest::SetSourceCode { code } => {
                            use mu::traits::ComputedHash;
                            let items = to_slint::split_source_code(&code);
                            main_window.set_source_code_model(to_slint::model_rc_from_items(items));

                            main_window.set_state(VM_State {
                                current_source_hash: hash_to_shared_string(code.computed_hash()),
                                current_line: 1,
                            });
                            main_window.set_source_code(code.inner_ref().into());
                        }
                        ViewModelRequest::SetSymbolTree(items) => {
                            main_window.set_symbol_tree(to_slint::model_rc_from_items(items))
                        }
                        ViewModelRequest::SetModelTree(items) => {
                            main_window.set_model_tree(to_slint::model_rc_from_items(items))
                        }
                        ViewModelRequest::SetCurrentLine(line) => {
                            let mut state = main_window.get_state();
                            state.current_line = line as i32;
                            main_window.set_state(state);
                        }
                    })
                    .expect("No error");
                }
            }
        });

        let viewer_process = self.viewer_process.clone();
        main_window.on_button_launch_viewer_clicked(move || {
            match viewer_process.write() {
                Ok(mut process) => {
                    *process = Some(ViewerProcessInterface::run(false));
                    log::warn!("Already running!");
                }
                Err(err) => log::error!("{err}"),
            };
        });

        let weak = main_window.as_weak();
        let viewer_process = self.viewer_process.clone();
        main_window.on_button_send_source_code_clicked(move || {
            let code = weak
                .upgrade()
                .expect("MainWindow component")
                .get_source_code()
                .to_string();
            log::info!("Send source code: {code}");

            match viewer_process.read() {
                Ok(process) => match &*process {
                    Some(process) => {
                        log::info!("Viewer request");
                        process
                            .send_request(ViewerRequest::ShowSourceCode {
                                path: Some(std::path::PathBuf::from(&input)),
                                name: None,
                                code,
                            })
                            .expect("No error");
                    }
                    None => {
                        log::error!("Process is not running!");
                    }
                },
                Err(err) => log::error!("{err}"),
            }
        });

        main_window.run().into_diagnostic()?;

        Ok(())
    }
}

fn main() -> miette::Result<()> {
    env_logger::init();

    Inspector::new()?.run()
}
