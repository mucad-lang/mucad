// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The µcad toolchain for WebAssembly: compile µcad code, render it and export it,
//! e.g. in a browser. See the README for the usage from JavaScript.

use std::sync::Arc;

mod completion;
mod diagnostic;
mod geometry;
mod literal;
mod share;

pub use completion::Completion;
pub use diagnostic::{Diagnostic, Severity};
pub use literal::{LiteralRange, SteppedLiteral};
pub use share::SharedLink;

use mu::{
    base::DiagRenderOptions,
    core::TriangleMesh,
    export::{Export, stl::StlExporter, svg::SvgExporter},
    render::{GeometryOutputs, RenderCache, Renderer},
    traits::RenderDiag,
    types::{ModelType, model::NodeExt},
};
use mucad_driver::{Compiler, prelude as mu};
use wasm_bindgen::prelude::*;

/// The library search path of the standard library in the virtual file system.
const LIB_PATH: &str = "/lib";

/// Diagnostics are rendered without colors, e.g. for a text area.
const DIAG_OPTIONS: DiagRenderOptions = DiagRenderOptions {
    color: false,
    unicode: true,
};

/// The µcad toolchain: compiles µcad code, renders and exports its model.
///
/// The standard library is embedded, there is no file system access.
#[wasm_bindgen]
pub struct Mucad {
    compiler: Compiler,
    renderer: Renderer,
    /// The model of the last successful compilation.
    model: Option<mu::ModelTree>,
}

#[wasm_bindgen]
impl Mucad {
    /// A toolchain with the embedded standard library and a render cache.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Mucad {
        console_error_panic_hook::set_once();

        let file_system = mu::stdlib::StdLib::virtual_file_system(LIB_PATH);
        let compiler = Compiler::from_config(&mu::DriverConfig {
            search_paths: vec![LIB_PATH.into()],
            ..Default::default()
        })
        .with_file_system(Arc::new(file_system));

        Self {
            compiler,
            renderer: Renderer::new().with_cache(mu::SharedMut::new(RenderCache::new())),
            model: None,
        }
    }

    /// Set the linear render resolution in millimeters, e.g. `0.1` for fine circles.
    #[wasm_bindgen(js_name = setResolution)]
    pub fn set_resolution(&mut self, linear: f64) {
        self.renderer.resolution = mu::RenderResolution { linear };
    }

    /// Compile µcad `code`; the model is kept for rendering and exporting.
    pub fn compile(&mut self, code: &str) -> CompileResult {
        let source = mu::Source::from(code);
        let (model, diagnostics) = match self.compiler.compile(source.clone()) {
            Ok((model, diags)) if !diags.has_errors() => (Some(model), diags),
            Ok((_, diags)) | Err(diags) => (None, diags),
        };
        let model_type = match model
            .as_ref()
            .map(|model| model.root().deduce_output_type())
        {
            Some(ModelType::Geometry2D) => "2d",
            Some(ModelType::Geometry3D) => "3d",
            Some(ModelType::Any) => "mixed",
            Some(ModelType::NotDetermined) | None => "none",
        };

        self.model = model;
        CompileResult {
            success: self.model.is_some(),
            diagnostics: diagnostics
                .iter()
                .map(|diagnostic| Diagnostic::new(diagnostic, &source))
                .collect(),
            report: self
                .compiler
                .render_diag_to_string_with(&diagnostics, &DIAG_OPTIONS),
            model_type: model_type.into(),
        }
    }

    /// The symbols to complete the path in front of a position in `code`, e.g. the symbols of
    /// `std::geo2d` for `std::geo2d::Ci`, or the visible names for `Ci`, sorted by name.
    ///
    /// The line and column start at 1, the column is counted in UTF-16 code units like in
    /// JavaScript. The symbols are those of the last compilation, also of another version of
    /// the code, e.g. while typing.
    pub fn complete(&self, code: &str, line_no: u32, column: u32) -> Vec<Completion> {
        let source = mu::Source::from(code);
        let offset = completion::offset(code, line_no, column);
        (self.compiler.completions(&source, offset).into_iter())
            .map(Completion::from)
            .collect()
    }

    /// The `literal` stepped by `factor` (e.g. `1`, `-10`) times the place of the digit at
    /// `cursor`, e.g. `12.4mm` for `12.3mm` with the cursor in front of `3`, `undefined` if it is
    /// no quantity, integer or bool literal.
    ///
    /// Without a digit at the cursor, the last digit is stepped, e.g. with the cursor on the
    /// unit, a bool is toggled. The decimals are kept. The cursor and the stepped digit are
    /// counted in UTF-16 code units from the start of the literal, like in JavaScript.
    #[wasm_bindgen(js_name = stepLiteralAt)]
    pub fn step_literal_at(literal: &str, cursor: u32, factor: f64) -> Option<SteppedLiteral> {
        literal::step_at(literal, cursor, factor)
    }

    /// The range of the literal at a position in `code`, e.g. under the cursor, also right
    /// after it, `undefined` if there is none.
    ///
    /// The line and column start at 1, the column is counted in UTF-16 code units like in
    /// JavaScript.
    #[wasm_bindgen(js_name = literalAt)]
    pub fn literal_at(code: &str, line_no: u32, column: u32) -> Option<LiteralRange> {
        literal::literal_at(code, line_no, column)
    }

    /// Format source code. Returns new code as string if parsing has been successful.
    pub fn format(&mut self, code: &str) -> FormatResult {
        match mu::format(code, &mu::FormatConfig::default()) {
            Ok(formatted) => FormatResult {
                success: true,
                formatted: formatted.formatted,
                code: formatted.source.code().to_string(),
            },
            Err(_) => FormatResult {
                success: false,
                formatted: false,
                code: String::new(),
            },
        }
    }

    /// Render a 3D model into triangle meshes, one for each color.
    #[wasm_bindgen(js_name = renderMeshes)]
    pub fn render_meshes(&mut self) -> Result<Vec<geometry::Mesh>, String> {
        Ok(geometry::by_color(self.render()?)
            .into_iter()
            .map(|(color, outputs)| {
                let mut mesh = geometry::Mesh::from(TriangleMesh::from(&outputs.to_3d()));
                if let Some(color) = color {
                    mesh = mesh.with_color(color);
                }
                mesh
            })
            .collect())
    }

    /// Render a 2D model into sketches, one for each color.
    #[wasm_bindgen(js_name = renderSketches)]
    pub fn render_sketches(&mut self) -> Result<Vec<geometry::Sketch>, String> {
        Ok(geometry::by_color(self.render()?)
            .into_iter()
            .map(|(color, outputs)| geometry::Sketch::new(color, &outputs))
            .collect())
    }

    /// Export a 3D model as binary STL.
    #[wasm_bindgen(js_name = exportStl)]
    pub fn export_stl(&self) -> Result<Vec<u8>, String> {
        self.export(&StlExporter {
            resolution: self.renderer.resolution.clone(),
            binary: true,
        })
    }

    /// Export a 2D model as SVG.
    #[wasm_bindgen(js_name = exportSvg)]
    pub fn export_svg(&self) -> Result<String, String> {
        let svg = self.export(&SvgExporter {
            resolution: self.renderer.resolution.clone(),
            ..Default::default()
        })?;
        String::from_utf8(svg).map_err(|err| err.to_string())
    }

    #[wasm_bindgen(js_name = versionString)]
    pub fn version_string() -> String {
        mu::MUCAD_VERSION.to_string()
    }

    /// The formatted µcad code of a shared link, e.g. of
    /// `https://play.mucad.org/#wrUBQTwABENpcmNsZShyID0gMW1tKTsD`, code with errors is unformatted.
    ///
    /// The link may be complete, with the code in its path, after `#` or as `?code=`,
    /// or only the link data, e.g. `location.hash`. Fails, e.g. for an invalid link.
    #[wasm_bindgen(js_name = decodeLink)]
    pub fn decode_link(link: &str) -> Result<String, String> {
        mu::share::decode(link).map_err(|err| err.to_string())
    }

    /// The link to the web viewer with the µcad `code`, with `strip` without comments.
    ///
    /// Fails, e.g. for code with more than 2048 bytes.
    #[wasm_bindgen(js_name = encodeLink)]
    pub fn encode_link(code: &str, strip: bool) -> Result<String, String> {
        mu::share::link(code, strip).map_err(|err| err.to_string())
    }

    /// The code and the preamble of a shared link, see `decodeLink`.
    ///
    /// The preamble is a JSON object, e.g. `{"name":"brick.mu"}`, `{}` without a preamble.
    #[wasm_bindgen(js_name = decodeLinkWithPreamble)]
    pub fn decode_link_with_preamble(link: &str) -> Result<SharedLink, String> {
        share::decode(link)
    }

    /// The link to the web viewer with the µcad `code` behind a `preamble`, see `encodeLink`.
    ///
    /// The preamble is a JSON object, e.g. `{"name":"brick.mu"}`, which is written as TOML,
    /// `{}` for no preamble. Fails, e.g. for values, which TOML has not, like `null`.
    #[wasm_bindgen(js_name = encodeLinkWithPreamble)]
    pub fn encode_link_with_preamble(
        code: &str,
        preamble: &str,
        strip: bool,
    ) -> Result<String, String> {
        share::encode(code, preamble, strip)
    }

    /// The modules of the QR code of `text` row by row, 1 for a dark module, without the
    /// quiet zone. The QR code is square, its width is the square root of the length.
    ///
    /// Fails, if the text is too long for a QR code.
    #[wasm_bindgen(js_name = qrCode)]
    pub fn qr_code(text: &str) -> Result<Vec<u8>, String> {
        // The lowest error correction holds the most data, e.g. the longest link.
        let code = qrcode::QrCode::with_error_correction_level(text, qrcode::EcLevel::L)
            .map_err(|err| err.to_string())?;
        Ok(code
            .to_colors()
            .into_iter()
            .map(|color| (color == qrcode::Color::Dark) as u8)
            .collect())
    }
}

impl Default for Mucad {
    fn default() -> Self {
        Self::new()
    }
}

impl Mucad {
    /// The model of the last successful compilation.
    fn model(&self) -> Result<&mu::ModelTree, String> {
        self.model
            .as_ref()
            .ok_or_else(|| "No model, compile without errors first".into())
    }

    /// Render the model, render errors are returned as diagnostics.
    fn render(&mut self) -> Result<GeometryOutputs, String> {
        let model = self
            .model
            .as_ref()
            .ok_or("No model, compile without errors first")?;
        match self.renderer.render(model) {
            Ok((tree, issues)) if !issues.has_errors() => Ok(tree.root().outputs.clone()),
            Ok((_, issues)) | Err(issues) => Err(self
                .renderer
                .render_diag_to_string_with(&issues.into_diagnostics(), &DIAG_OPTIONS)),
        }
    }

    /// Export the model with `exporter` into memory.
    fn export(&self, exporter: &dyn Export) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        exporter
            .export(self.model()?, &mut buffer)
            .map_err(|err| err.to_string())?;
        Ok(buffer)
    }
}

#[wasm_bindgen]
pub struct FormatResult {
    success: bool,
    formatted: bool,
    code: String,
}

#[wasm_bindgen]
impl FormatResult {
    /// `true` if the code compiled without errors, so the model can be rendered and exported.
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    /// `true` if the code compiled without errors, so the model can be rendered and exported.
    #[wasm_bindgen(getter)]
    pub fn formatted(&self) -> bool {
        self.formatted
    }

    /// `true` if the code compiled without errors, so the model can be rendered and exported.
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }
}

/// The result of a compilation.
#[wasm_bindgen]
pub struct CompileResult {
    success: bool,
    diagnostics: Vec<Diagnostic>,
    report: String,
    model_type: String,
}

#[wasm_bindgen]
impl CompileResult {
    /// `true` if the code compiled without errors, so the model can be rendered and exported.
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Errors, warnings and advices with their positions in the code.
    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }

    /// Errors, warnings and advices as text, with the code they refer to.
    #[wasm_bindgen(getter)]
    pub fn report(&self) -> String {
        self.report.clone()
    }

    /// The type of the model: `"2d"`, `"3d"`, `"mixed"` or `"none"`.
    #[wasm_bindgen(getter, js_name = modelType)]
    pub fn model_type(&self) -> String {
        self.model_type.clone()
    }
}
