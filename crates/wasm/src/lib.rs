// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The µcad toolchain for WebAssembly: compile µcad code, render it and export it,
//! e.g. in a browser. See the README for the usage from JavaScript.

use std::sync::Arc;

mod diagnostic;
mod geometry;

pub use diagnostic::{Diagnostic, Severity};

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
