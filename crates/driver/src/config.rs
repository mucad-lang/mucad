// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad driver config.

use miette::IntoDiagnostic;
use mucad_render::RenderResolution;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DriverConfig {
    pub no_std: bool,

    pub search_paths: Vec<std::path::PathBuf>,

    /// Default extension (default: `µcad`).
    pub default_extension: String,

    /// Export settings.
    pub export: ExporterConfig,

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
            search_paths: vec![/*mucad_std::global_library_search_path()*/],
        }
    }
}

impl DriverConfig {
    /// Load config from TOML file.
    pub fn load(filename: &std::path::Path) -> miette::Result<Self> {
        let content = std::fs::read_to_string(filename).into_diagnostic()?;
        let mut config: DriverConfig = toml::from_str(&content).into_diagnostic()?;

        if !mucad_lang_base::MUCAD_EXTENSIONS.contains(&config.default_extension.as_str()) {
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
pub struct ExporterConfig {
    /// Default sketch exporter.
    pub sketch: String,
    /// Default part exporter.
    pub part: String,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            sketch: "svg".into(),
            part: "stl".into(),
        }
    }
}

impl ExporterConfig {
    pub fn render_resolution(&self) -> RenderResolution {
        todo!()
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct FormatConfig;

impl From<&FormatConfig> for mucad_lang_format::FormatConfig {
    fn from(_: &FormatConfig) -> Self {
        mucad_lang_format::FormatConfig::default()
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct DiagnosticsConfig;

impl From<&DiagnosticsConfig> for mucad_lang_base::DiagRenderOptions {
    fn from(_: &DiagnosticsConfig) -> Self {
        mucad_lang_base::DiagRenderOptions::default()
    }
}
