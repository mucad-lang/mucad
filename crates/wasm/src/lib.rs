// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The µcad toolchain for WebAssembly: compile µcad code, render it and export it,
//! e.g. in a browser. See the README for the usage from JavaScript.

use std::sync::Arc;

use mu::{
    base::DiagRenderOptions,
    core::{Color, Geometry2D, MultiPolygon, TriangleMesh},
    export::{Export, stl::StlExporter, svg::SvgExporter},
    render::{GeometryOutput, GeometryOutputs, RenderCache, Renderer},
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

    /// Render a 3D model into triangle meshes, one for each color.
    #[wasm_bindgen(js_name = renderMeshes)]
    pub fn render_meshes(&mut self) -> Result<Vec<Mesh>, String> {
        Ok(by_color(self.render()?)
            .into_iter()
            .map(|(color, outputs)| Mesh {
                color: color.map(rgba),
                ..TriangleMesh::from(&outputs.to_3d()).into()
            })
            .collect())
    }

    /// Render a 2D model into sketches, one for each color.
    #[wasm_bindgen(js_name = renderSketches)]
    pub fn render_sketches(&mut self) -> Result<Vec<Sketch>, String> {
        Ok(by_color(self.render()?)
            .into_iter()
            .map(|(color, outputs)| Sketch::new(color, &outputs))
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

/// The outputs grouped by their color, in the order of the first output of each color.
fn by_color(outputs: GeometryOutputs) -> Vec<(Option<Color>, GeometryOutputs)> {
    let mut groups: Vec<(Option<Color>, Vec<Arc<GeometryOutput>>)> = Vec::new();
    for output in outputs {
        let color = output.attr.fill_color;
        match groups
            .iter_mut()
            .find(|(group_color, _)| *group_color == color)
        {
            Some((_, group)) => group.push(output),
            None => groups.push((color, vec![output])),
        }
    }
    groups
        .into_iter()
        .map(|(color, group)| (color, GeometryOutputs::from_iter(group)))
        .collect()
}

/// A color as `r, g, b, a` from `0.0` to `1.0`.
fn rgba(color: Color) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
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

/// The severity of a diagnostic.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

/// An error, warning or advice with its range in the code, e.g. for an editor marker.
///
/// Lines and columns start at 1, columns are counted in UTF-16 code units like in JavaScript.
/// A diagnostic without position in the code, e.g. of the standard library, is at its start.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Diagnostic {
    message: String,
    start_line_no: u32,
    start_column: u32,
    end_line_no: u32,
    end_column: u32,
    severity: Severity,
}

#[wasm_bindgen]
impl Diagnostic {
    /// The message with a help, if there is one.
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// The line of the start, from 1.
    #[wasm_bindgen(getter, js_name = startLineNo)]
    pub fn start_line_no(&self) -> u32 {
        self.start_line_no
    }

    /// The column of the start, from 1.
    #[wasm_bindgen(getter, js_name = startColumn)]
    pub fn start_column(&self) -> u32 {
        self.start_column
    }

    /// The line of the end, from 1.
    #[wasm_bindgen(getter, js_name = endLineNo)]
    pub fn end_line_no(&self) -> u32 {
        self.end_line_no
    }

    /// The column after the end, from 1.
    #[wasm_bindgen(getter, js_name = endColumn)]
    pub fn end_column(&self) -> u32 {
        self.end_column
    }

    /// Whether this is an error, a warning or an advice.
    #[wasm_bindgen(getter)]
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

impl Diagnostic {
    /// The diagnostic of the compiler with its range in `source`, the compiled code.
    ///
    /// The code is needed for the end of the range and for the columns in UTF-16 code units.
    fn new(diagnostic: &mu::Diagnostic, source: &mu::Source) -> Self {
        let code = source.code();
        let src_ref = &diagnostic.src_ref;
        let (start, end) = if src_ref.is_some() && src_ref.source_hash() == source.hash_id() {
            (src_ref.start, src_ref.end)
        } else {
            (0, 0)
        };
        let (start_line_no, start_column) = line_column(code, start);
        let (end_line_no, end_column) = line_column(code, end);

        Self {
            message: match diagnostic.help() {
                Some(help) => format!("{}\n{help}", diagnostic.report),
                None => diagnostic.report.to_string(),
            },
            start_line_no,
            start_column,
            end_line_no,
            end_column,
            severity: match diagnostic.severity() {
                Some(mu::base::Severity::Advice) => Severity::Advice,
                Some(mu::base::Severity::Warning) => Severity::Warning,
                Some(mu::base::Severity::Error) | None => Severity::Error,
            },
        }
    }
}

/// The line and column (from 1, in UTF-16 code units) of the byte `offset` in `code`.
fn line_column(code: &str, offset: usize) -> (u32, u32) {
    let before = code.get(..offset).unwrap_or(code);
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    (
        before.matches('\n').count() as u32 + 1,
        before[line_start..].encode_utf16().count() as u32 + 1,
    )
}

/// A triangle mesh, e.g. for a `THREE.BufferGeometry`.
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Mesh {
    positions: Vec<f32>,
    indices: Vec<u32>,
    color: Option<[f32; 4]>,
}

#[wasm_bindgen]
impl Mesh {
    /// The color as `r, g, b, a` from `0.0` to `1.0`, e.g. of `#[color = "red"]`,
    /// or `undefined` for the default color.
    #[wasm_bindgen(getter)]
    pub fn color(&self) -> Option<Vec<f32>> {
        self.color.map(Vec::from)
    }

    /// The vertex positions as `x, y, z` triples.
    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> Vec<f32> {
        self.positions.clone()
    }

    /// The vertex indices of the triangles as triples.
    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
}

impl Mesh {
    /// Triangulate polygons into a mesh in the XY plane.
    fn triangulate(multi_polygon: &MultiPolygon) -> Self {
        use geo::TriangulateEarcut;

        let mut mesh = Self::default();
        for polygon in multi_polygon {
            let triangulation = polygon.earcut_triangles_raw();
            let offset = (mesh.positions.len() / 3) as u32;
            mesh.positions.extend(
                triangulation
                    .vertices
                    .iter()
                    .flat_map(|[x, y]| [*x as f32, *y as f32, 0.0]),
            );
            mesh.indices.extend(
                triangulation
                    .triangle_indices
                    .iter()
                    .map(|index| offset + *index as u32),
            );
        }
        mesh
    }
}

impl From<TriangleMesh> for Mesh {
    fn from(mesh: TriangleMesh) -> Self {
        Self {
            positions: mesh
                .positions
                .iter()
                .flat_map(|position| [position.x, position.y, position.z])
                .collect(),
            indices: mesh
                .triangle_indices
                .iter()
                .flat_map(|triangle| [triangle.0, triangle.1, triangle.2])
                .collect(),
            color: None,
        }
    }
}

/// The 2D geometries of a model with one color, in several forms.
#[wasm_bindgen]
pub struct Sketch {
    polygons: Polygons,
    lines: Lines,
    triangles: Mesh,
    color: Option<[f32; 4]>,
}

#[wasm_bindgen]
impl Sketch {
    /// The polygons with holes, e.g. for a canvas or SVG.
    #[wasm_bindgen(getter)]
    pub fn polygons(&self) -> Polygons {
        self.polygons.clone()
    }

    /// The outlines as line segments, e.g. for a `THREE.LineSegments`.
    #[wasm_bindgen(getter)]
    pub fn lines(&self) -> Lines {
        self.lines.clone()
    }

    /// The filled areas as triangle mesh in the XY plane, e.g. for a `THREE.Mesh`.
    #[wasm_bindgen(getter)]
    pub fn triangles(&self) -> Mesh {
        self.triangles.clone()
    }

    /// The color as `r, g, b, a` from `0.0` to `1.0`, e.g. of `#[color = "red"]`,
    /// or `undefined` for the default color.
    #[wasm_bindgen(getter)]
    pub fn color(&self) -> Option<Vec<f32>> {
        self.color.map(Vec::from)
    }
}

impl Sketch {
    fn new(color: Option<Color>, outputs: &GeometryOutputs) -> Self {
        let geometries = outputs.to_2d();
        let mut lines = Lines::default();
        geometries.iter().for_each(|geometry| lines.add(geometry));
        let multi_polygon = MultiPolygon::from(geometries);
        Self {
            triangles: Mesh::triangulate(&multi_polygon),
            polygons: multi_polygon.into(),
            lines,
            color: color.map(rgba),
        }
    }
}

/// Polygons with holes.
///
/// The points of all rings are stored one after another in `coordinates`.
/// `rings` and `polygons` contain start indices with an additional end index:
/// ring `i` consists of the points `rings[i]..rings[i + 1]`,
/// polygon `j` of the rings `polygons[j]..polygons[j + 1]`, the first one is its outline,
/// the others are its holes.
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Polygons {
    coordinates: Vec<f32>,
    rings: Vec<u32>,
    polygons: Vec<u32>,
}

#[wasm_bindgen]
impl Polygons {
    /// The points of all rings as `x, y` pairs.
    #[wasm_bindgen(getter)]
    pub fn coordinates(&self) -> Vec<f32> {
        self.coordinates.clone()
    }

    /// The index of the first point of each ring and the number of points.
    #[wasm_bindgen(getter)]
    pub fn rings(&self) -> Vec<u32> {
        self.rings.clone()
    }

    /// The index of the first ring of each polygon and the number of rings.
    #[wasm_bindgen(getter)]
    pub fn polygons(&self) -> Vec<u32> {
        self.polygons.clone()
    }
}

impl From<MultiPolygon> for Polygons {
    fn from(multi_polygon: MultiPolygon) -> Self {
        let mut polygons = Self::default();
        for polygon in &multi_polygon {
            polygons.polygons.push(polygons.rings.len() as u32);
            for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
                polygons.rings.push((polygons.coordinates.len() / 2) as u32);
                polygons
                    .coordinates
                    .extend(ring.coords().flat_map(|c| [c.x as f32, c.y as f32]));
            }
        }
        polygons.rings.push((polygons.coordinates.len() / 2) as u32);
        polygons.polygons.push(polygons.rings.len() as u32 - 1);
        polygons
    }
}

/// Line segments, e.g. for a `THREE.LineSegments`.
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Lines {
    positions: Vec<f32>,
}

#[wasm_bindgen]
impl Lines {
    /// The start and end points of the segments as `x, y` pairs.
    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> Vec<f32> {
        self.positions.clone()
    }
}

impl Lines {
    /// Add the segments of a line string or of the outlines of polygons.
    fn add(&mut self, geometry: &Geometry2D) {
        match geometry {
            Geometry2D::LineString(line_string) => self.add_line_string(line_string),
            Geometry2D::MultiLineString(multi_line_string) => multi_line_string
                .iter()
                .for_each(|line_string| self.add_line_string(line_string)),
            Geometry2D::Line(line) => self.positions.extend([
                line.0.x() as f32,
                line.0.y() as f32,
                line.1.x() as f32,
                line.1.y() as f32,
            ]),
            Geometry2D::Polygon(polygon) => self.add_polygon(polygon),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon
                .iter()
                .for_each(|polygon| self.add_polygon(polygon)),
            Geometry2D::Collection(geometries) => {
                geometries.iter().for_each(|geometry| self.add(geometry))
            }
        }
    }

    fn add_polygon(&mut self, polygon: &geo::Polygon) {
        std::iter::once(polygon.exterior())
            .chain(polygon.interiors())
            .for_each(|ring| self.add_line_string(ring));
    }

    fn add_line_string(&mut self, line_string: &geo::LineString) {
        self.positions.extend(
            line_string
                .lines()
                .flat_map(|line| [line.start.x, line.start.y, line.end.x, line.end.y])
                .map(|value| value as f32),
        );
    }
}
