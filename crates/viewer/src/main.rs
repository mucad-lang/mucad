// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
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

use microcad_driver::prelude as mu;

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

    /// Paths to search for files.
    ///
    /// By default, `./std/lib` (if it exists) and `~/.microcad/lib` are used.
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
                    "file" => Ok(Some(mu::locate::to_url(input)?)),
                    scheme => Err(miette::miette!("Unknown scheme: {scheme}")),
                }
            }
            None => Ok(None),
        }
    }
}

use microcad_viewer::Config;

use microcad_viewer::MicrocadPlugin;
use url::Url;

fn main() {
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
        ..Default::default()
    };

    use microcad_viewer::plugin::MicrocadPluginInput;

    let mut app = App::new();
    app
        // Power-saving reactive rendering for applications.
        .insert_resource(bevy::winit::WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_millis(1000 / 60)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_millis(1000 / 10)),
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(MicrocadPlugin {
            input: url.map(|url| MicrocadPluginInput::from_url(url).expect("Valid URL")),
            config,
        });

    // Workaround for flickering entity bug on Intel GPUs:
    // https://github.com/bevyengine/bevy/issues/18904
    app.sub_app_mut(RenderApp)
        .insert_resource(GpuPreprocessingSupport {
            max_supported_mode: GpuPreprocessingMode::None,
        });

    app.run();
}
