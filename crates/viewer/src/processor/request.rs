// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Viewer processor request.

use crate::Config;

use mucad_driver::prelude as mu;

/// A processor request.
///
/// Commands that can be passed to the [`Processor`].
#[derive(Clone)]
pub enum ProcessorRequest {
    /// Initialize the interpreter.
    ///
    /// Request must only be sent once and sets the initialize flag to `true`.
    Initialize {
        /// The initial config
        config: Config,
    },
    /// Compile a file.
    CompileFile(std::path::PathBuf),
    /// Compile some source code.
    CompileSource {
        /// Virtual file path
        path: Option<std::path::PathBuf>,
        /// Optional name of the source code snippet, e.g. the full file name.
        name: Option<String>,
        /// The actual source code.
        source: String,
    },
    /// Render the geometry. This message should be sent when the source code has been modified.
    Render(Option<mu::RenderResolution>),
    /// Export the geometry to a file.
    Export {
        /// File name.
        filename: std::path::PathBuf,
        /// Optional exporter ("svg", "stl").
        exporter: Option<String>,
    },
    /// Highlight models with a certain line number, if the line number is not [`None`].
    ///
    /// Rerenders the model.
    SetLineNumber(Option<u32>),
}
