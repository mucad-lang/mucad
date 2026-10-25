// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad viewer

use std::time::Duration;

use bevy::{
    DefaultPlugins,
    app::App,
    render::{
        RenderApp,
        batching::gpu_preprocessing::{GpuPreprocessingMode, GpuPreprocessingSupport},
    },
    winit::UpdateMode,
};
use clap::Parser;
use miette::IntoDiagnostic;

use mucad_driver::prelude as mu;

/// µcad cli
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The input URL defines the resource to be displayed.
    ///
    /// Examples:
    /// * `my_file.mu` Display contents in file.
    /// * `my_file.mu?symbol=MyPart#L11`: Display some symbol `MyPart` at line 11.
    /// * `stdin://`: Read from stdin.
    input: Option<String>,

    /// Windows stays on top.
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    stay_on_top: bool,

    /// Windows stays hidden (and can be shown via IPC.
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    hidden: bool,

    /// Verbosity level (use -v, -vv, or -vvv)
    #[arg(short, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Save a screenshot of the model into a PNG file and exit, e.g. `--screenshot box.png`.
    #[arg(long, value_name = "PNG")]
    screenshot: Option<std::path::PathBuf>,

    /// The direction from the model to the camera, e.g. `1,-1,1` from the front right above.
    #[arg(long, value_name = "X,Y,Z", value_parser = parse_direction, default_value = "1,-1,1")]
    cam: bevy::math::Vec3,

    /// The zoom, e.g. `100%` shows the whole scene, `200%` the model twice as big.
    #[arg(long, value_name = "PERCENT", value_parser = parse_zoom, default_value = "100%")]
    zoom: f32,

    /// Paths to search for files.
    ///
    /// By default, `./std/lib` (if it exists) and `~/.mucad/lib` are used.
    #[arg(short = 'P', long = "search-path", action = clap::ArgAction::Append, global = true)]
    pub search_paths: Vec<std::path::PathBuf>,
}

impl Args {
    /// Return input string as URL.
    fn input_as_url(&self) -> miette::Result<Option<Url>> {
        match &self.input {
            Some(input) => {
                let (scheme, _) = input.split_once("://").unwrap_or(("file", input));
                match scheme {
                    "stdin" => {
                        let url = Url::parse("stdin://").into_diagnostic()?;
                        Ok(Some(url))
                    }
                    "file" => Ok(Some(mu::locate::to_url(
                        &mu::resolve::LocalFileSystem,
                        input,
                    )?)),
                    scheme => Err(miette::miette!("Unknown scheme: {scheme}")),
                }
            }
            None => Ok(None),
        }
    }
}

use mucad_viewer::Config;

/// Parse a direction, e.g. `1.0,1.0,0.7`.
fn parse_direction(text: &str) -> Result<bevy::math::Vec3, String> {
    let values = text
        .split(',')
        .map(|value| value.trim().parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("{err}, e.g. `1,-1,1`"))?;
    match values[..] {
        [x, y, z] if x != 0.0 || y != 0.0 || z != 0.0 => Ok(bevy::math::Vec3::new(x, y, z)),
        [_, _, _] => Err("The direction must not be zero".into()),
        _ => Err("Three values expected, e.g. `1,-1,1`".into()),
    }
}

/// Parse a zoom in percent, e.g. `200%`, or as factor, e.g. `2.0`.
fn parse_zoom(text: &str) -> Result<f32, String> {
    let zoom = match text.trim().strip_suffix('%') {
        Some(percent) => percent.trim().parse::<f32>().map(|percent| percent / 100.0),
        None => text.trim().parse::<f32>(),
    }
    .map_err(|err| format!("{err}, e.g. `200%`"))?;
    if zoom > 0.0 {
        Ok(zoom)
    } else {
        Err("The zoom must be positive".into())
    }
}

use mucad_viewer::MucadPlugin;
use url::Url;

fn main() -> std::process::ExitCode {
    // Parse the command-line args before starting the app
    let args = Args::parse();

    // Initialize env_logger with a default filter level
    env_logger::Builder::from_default_env()
        .filter_level(match args.verbose {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            3 => log::LevelFilter::Trace,
            _ => panic!("unknown verbosity level"),
        }) // Set the default log level
        .init();

    let url = args.input_as_url();
    let url = match &url {
        Ok(url) => url.clone(),
        Err(err) => {
            log::error!(
                "{err} ({input})",
                input = match &args.input {
                    Some(input) => format!("({input})"),
                    None => String::new(),
                }
            );
            None
        }
    };

    let config = Config {
        search_paths: args.search_paths,
        stay_on_top: args.stay_on_top,
        camera_view: mucad_viewer::scene::CameraView {
            direction: args.cam,
            zoom: args.zoom,
        },
        screenshot: args.screenshot,
        ..Default::default()
    };

    use mucad_viewer::plugin::MucadPluginInput;

    let mut app = App::new();
    app
        // Power-saving reactive rendering for applications.
        .insert_resource(bevy::winit::WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_millis(1000 / 60)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_millis(1000 / 10)),
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(MucadPlugin {
            input: url.map(|url| MucadPluginInput::from_url(url).expect("Valid URL")),
            config,
        });

    // Workaround for flickering entity bug on Intel GPUs:
    // https://github.com/bevyengine/bevy/issues/18904
    app.sub_app_mut(RenderApp)
        .insert_resource(GpuPreprocessingSupport {
            max_supported_mode: GpuPreprocessingMode::None,
        });

    // E.g. an error, if a screenshot could not be taken.
    match app.run() {
        bevy::app::AppExit::Success => std::process::ExitCode::SUCCESS,
        bevy::app::AppExit::Error(code) => std::process::ExitCode::from(code.get()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_arguments() {
        assert_eq!(
            parse_direction("1.0,1.0,0.7"),
            Ok(bevy::math::Vec3::new(1.0, 1.0, 0.7))
        );
        assert!(parse_direction("1,2").is_err());
        assert!(parse_direction("0,0,0").is_err());

        assert_eq!(parse_zoom("100%"), Ok(1.0));
        assert_eq!(parse_zoom("250%"), Ok(2.5));
        assert_eq!(parse_zoom("0.5"), Ok(0.5));
        assert!(parse_zoom("0%").is_err());
        assert!(parse_zoom("big").is_err());
    }
}
