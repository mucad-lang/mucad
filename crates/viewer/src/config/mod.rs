// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Viewer settings.

pub mod theme;

use std::time::Duration;
pub use theme::Theme;

use bevy::ecs::resource::Resource;

use mucad_driver::prelude as mu;

/// Viewer configuration.
#[derive(Resource, serde::Deserialize, Clone)]
pub struct Config {
    /// Additional search paths for libraries.
    ///
    /// The standard library is found in its global path.
    pub search_paths: Vec<std::path::PathBuf>,

    /// Delay when the input file is reloaded.
    pub reload_delay: Duration,

    /// Window stays on top.
    pub stay_on_top: bool,

    /// Mesh smoothness threshold angle (default = 20°)
    pub mesh_smoothness_angle: mu::Scalar,

    /// Render resolution in mm (default = 0.25mm)
    pub render_resolution: mu::Scalar,

    /// Export resolution in mm (default = 0.1mm)
    pub export_resolution: mu::Scalar,

    /// The viewer theme.
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search_paths: Vec::new(),
            reload_delay: Duration::from_millis(100),
            stay_on_top: false,
            mesh_smoothness_angle: 20.0,
            render_resolution: mu::RenderResolution::medium().linear,
            export_resolution: mu::RenderResolution::high().linear,
            theme: Theme::default(),
        }
    }
}
