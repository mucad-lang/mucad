// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver config.

use microcad_render::RenderResolution;
use miette::IntoDiagnostic;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DriverConfig {
    pub no_std: bool,

    pub search_paths: Vec<std::path::PathBuf>,

    /// Default extension (default: `µcad`).
    pub default_extension: String,

    /// Export settings.
    pub export: ExportConfig,

    /// Format config
    pub format: FormatConfig,

    /// Diagnostics config
    pub diagnostics: DiagnosticsConfig,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            no_std: false,
            default_extension: "µcad".to_string(),
            export: Default::default(),
            format: Default::default(),
            diagnostics: Default::default(),
            search_paths: vec![/*microcad_std::global_library_search_path()*/],
        }
    }
}

impl DriverConfig {
    /// Load config from TOML file.
    pub fn load(filename: &std::path::Path) -> miette::Result<Self> {
        let content = std::fs::read_to_string(filename).into_diagnostic()?;
        let mut config: DriverConfig = toml::from_str(&content).into_diagnostic()?;

        if !microcad_lang_base::MUCAD_EXTENSIONS.contains(&config.default_extension.as_str()) {
            let fallback = DriverConfig::default().default_extension;
            log::warn!(
                "`{}` is a valid µcad extension, switching to `{fallback}`.",
                config.default_extension
            );
            config.default_extension = fallback;
        }

        Ok(config)
    }

    /// Return a path with default µcad extension given in the config.
    pub fn path_with_default_ext(&self, path: impl AsRef<std::path::Path>) -> std::path::PathBuf {
        let mut path = path.as_ref().to_path_buf();
        if path.extension().is_none() {
            path.set_extension(self.default_extension.clone());
        }
        path
    }
}

/// Export settings.
#[derive(Deserialize, Clone, Debug)]
pub struct ExportConfig {
    /// Default sketch exporter.
    pub sketch: String,
    /// Default part exporter.
    pub part: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            sketch: "svg".into(),
            part: "stl".into(),
        }
    }
}

impl ExportConfig {
    pub fn render_resolution(&self) -> RenderResolution {
        todo!()
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct FormatConfig;

impl From<&FormatConfig> for microcad_lang_format::FormatConfig {
    fn from(_: &FormatConfig) -> Self {
        microcad_lang_format::FormatConfig::default()
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct DiagnosticsConfig;

impl From<&DiagnosticsConfig> for microcad_lang_base::DiagRenderOptions {
    fn from(_: &DiagnosticsConfig) -> Self {
        microcad_lang_base::DiagRenderOptions::default()
    }
}
