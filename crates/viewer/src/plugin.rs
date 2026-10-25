// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! This module contains the mucad bevy plugin and its input interface.

use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use bevy::app::{App, Plugin, Startup, Update};
use bevy_mod_outline::OutlinePlugin;

use bevy::prelude::*;
use url::Url;

use crate::{stdin::StdinMessageReceiver, *};

/// Plugin input
///
/// The plugin can be represented as URL:
///
/// * File: file://my_file.mu?symbol=my::mod::MyPart#L11
/// * Stdin: stdin://
#[derive(Clone)]
pub enum MucadPluginInput {
    /// Load and watch an input file.
    File {
        /// File path that is loaded.
        path: std::path::PathBuf,

        /// Full name of resolved symbol to displayed, `std::geo2d::Rect`.
        ///
        /// If there is no symbol given, the source file will be used.
        symbol: Option<String>,

        /// Line number, starting with 1.
        line: Option<u32>,

        /// Time stamp of last modification.
        last_modified: Arc<Mutex<Option<SystemTime>>>,
    },
    /// Remove-controlled via stdin.
    Stdin(Option<StdinMessageReceiver>),
}

impl MucadPluginInput {
    /// Construct input from URL.
    pub fn from_url(url: Url) -> miette::Result<Self> {
        match url.scheme() {
            "file" => Ok(MucadPluginInput::File {
                path: url.to_file_path().expect("Valid file path").to_path_buf(),
                symbol: url.query_pairs().find_map(|(key, value)| {
                    if key == "symbol" {
                        Some(value.into())
                    } else {
                        None
                    }
                }),
                line: url
                    .fragment()
                    .and_then(|frag| frag.strip_prefix('L'))
                    .and_then(|n| n.parse::<u32>().ok()),
                last_modified: Default::default(),
            }),
            "stdin" => Ok(Self::Stdin(None)),
            scheme => Err(miette::miette!("{scheme} not supported!")),
        }
    }

    /// Get the URL for the input.
    pub fn get_url(&self) -> Url {
        match self {
            MucadPluginInput::File {
                path, symbol, line, ..
            } => {
                // Start with base: file://<path>
                let mut url = Url::parse("file://").expect("A valid URL");

                // PathBuf -> string (handle both relative and absolute)
                // Note: url::Url requires forward slashes
                let path_str = path.to_string_lossy().replace('\\', "/");
                url.set_path(&path_str);

                // Add symbol as query parameter if present
                if let Some(symbol) = symbol {
                    url.set_query(Some(&format!("symbol={}", symbol)));
                }

                // Add line as fragment (#L<number>) if present
                if let Some(line) = line {
                    url.set_fragment(Some(&format!("L{}", line)));
                }

                url
            }

            MucadPluginInput::Stdin(stdin) => {
                let default = Url::parse("stdin://").expect("No error");
                match stdin {
                    Some(stdin) => stdin.current_path().as_ref().map_or(default, |path| {
                        Url::parse(format!("stdin://{}", path.display()).as_str())
                            .expect("No error")
                    }),
                    None => default,
                }
            }
        }
    }

    fn display_url_human_readable(&self) -> String {
        // Decode the path
        let url = self.get_url();
        let path = percent_encoding::percent_decode_str(url.path()).decode_utf8_lossy();

        // Rebuild the visible parts
        let mut s = format!("{}://{}", url.scheme(), path);

        if let Some(q) = url.query() {
            s.push('?');
            s.push_str(q);
        }
        if let Some(frag) = url.fragment() {
            s.push('#');
            s.push_str(frag);
        }

        s
    }
}

impl std::fmt::Display for MucadPluginInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_url_human_readable())
    }
}

/// The mucad plugin.
pub struct MucadPlugin {
    /// The input for the plugin (e.g. file or stdin).
    pub input: Option<MucadPluginInput>,
    /// The viewer configuration.
    pub config: Config,
}

impl Plugin for MucadPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<view_model::ModelViewState>()
            .add_message::<view_model::ViewerEvent>()
            .insert_resource(ClearColor(self.config.theme.primary.to_bevy()))
            .insert_resource(ViewModel::new(self.input.clone(), self.config.clone()))
            .add_plugins(processor::ProcessorPlugin)
            .add_plugins(material::MaterialPlugin)
            .add_plugins(scene::ScenePlugin)
            .add_plugins(crate::screenshot::ScreenshotPlugin)
            .add_plugins((OutlinePlugin, MeshPickingPlugin))
            .add_systems(Startup, apply_window_settings)
            .add_systems(Update, stdin::handle_stdin_messages)
            .add_systems(Update, view_model::handle_viewer_event);
    }
}

fn apply_window_settings(state: Res<ViewModel>, mut windows: Query<&mut Window>) {
    match windows.single_mut() {
        Ok(mut window) => state.update_window_settings(&mut window),
        Err(e) => log::error!("{e}"),
    }
}
