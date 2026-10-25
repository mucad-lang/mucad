// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Save a screenshot of the model and exit, see [`crate::Config::screenshot`].

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};

use crate::{ViewModel, processor::ProcessingState};

/// The frames to render after the model has been spawned, before the screenshot is taken,
/// e.g. to upload the meshes and to adjust the camera to the size of the model.
const FRAMES_BEFORE_SCREENSHOT: u32 = 10;

/// The time to wait for a model, before the viewer exits without screenshot.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// The screenshot plugin takes a screenshot of the model, if the config has a path for it.
pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, take_screenshot);
    }
}

/// Take the screenshot, when the model has been rendered, and exit after saving it.
///
/// The viewer exits with an error, if the model has errors or does not appear in time.
fn take_screenshot(
    mut commands: Commands,
    state: Res<ViewModel>,
    time: Res<Time>,
    mut ready_frames: Local<u32>,
    mut taken: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = &state.config.screenshot else {
        return;
    };
    if *taken {
        return;
    }

    if matches!(state.processing_state, ProcessingState::Error) {
        eprintln!("Cannot take a screenshot, the model has errors");
        exit.write(AppExit::error());
        return;
    }
    // The model is ready, when its entities have been spawned. The processing state is not
    // used for that, because progress messages may arrive after the processor became idle.
    if !state.scene.model_entities.is_empty() {
        *ready_frames += 1;
    }
    if *ready_frames < FRAMES_BEFORE_SCREENSHOT {
        if time.elapsed() > TIMEOUT {
            eprintln!("Cannot take a screenshot, there is no model");
            exit.write(AppExit::error());
        }
        return;
    }

    *taken = true;
    let path = path.clone();
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| match save(
            &captured.image,
            &path,
        ) {
            Ok(()) => {
                eprintln!("Saved screenshot '{}'", path.display());
                exit.write(AppExit::Success);
            }
            Err(err) => {
                eprintln!("Cannot save screenshot '{}': {err}", path.display());
                exit.write(AppExit::error());
            }
        },
    );
}

/// Save `image` as PNG file without alpha channel.
fn save(image: &Image, path: &std::path::Path) -> Result<(), String> {
    image
        .clone()
        .try_into_dynamic()
        .map_err(|err| err.to_string())?
        .to_rgb8()
        .save(path)
        .map_err(|err| err.to_string())
}
