// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Viewer State module.

mod event;
mod model;

use bevy::{ecs::resource::Resource, window::Window};

use crate::{
    Config,
    plugin::MucadPluginInput,
    processor::{ProcessingState, ProcessorInterface},
    scene::Scene,
};

pub use event::{ViewerEvent, handle_viewer_event};
pub use model::ModelViewState;

/// TODO Cursor struct to infos about a cursor.
#[derive(Default)]
pub struct Cursor;

/// The application state (the bevy view model).
#[derive(Resource)]
pub struct ViewModel {
    /// Input interface (e.g. file or stdin).
    pub input: Option<MucadPluginInput>,
    /// The configuration settings of loaded at startup.
    pub config: Config,
    /// The scene entities to be spawned and rendered.
    pub scene: Scene,
    /// Information at cursor positions (view cursor and editor cursor).
    pub cursor: Cursor,
    /// The µcad geometry processor.
    pub processor: ProcessorInterface,
    /// The current processing state.
    pub processing_state: ProcessingState,
}

impl ViewModel {
    /// Create new state from arguments.
    pub fn new(input: Option<MucadPluginInput>, config: Config) -> Self {
        Self {
            input,
            config,
            cursor: Default::default(),
            scene: Default::default(),
            processor: ProcessorInterface::run(),
            processing_state: Default::default(),
        }
    }

    /// Update window title and window level.
    pub fn update_window_settings(&self, window: &mut Window) {
        window.title = format!(
            "µcad{}",
            match &self.input {
                Some(input) => format!(" - {input}"),
                None => String::new(),
            }
        );
        window.window_level = match self.config.stay_on_top {
            true => bevy::window::WindowLevel::AlwaysOnTop,
            false => bevy::window::WindowLevel::Normal,
        };
    }
}
