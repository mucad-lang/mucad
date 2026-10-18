// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The µcad toolchain for WebAssembly: compile µcad code, render it and export it,
//! e.g. in a browser. See the README for the usage from JavaScript.

use std::sync::Arc;

use mu::{
    base::DiagRenderOptions,
    core::{Geometry2D, MultiPolygon, TriangleMesh},
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
        let (model, diagnostics) = match self.compiler.compile(code) {
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
            diagnostics: self
                .compiler
                .render_diag_to_string_with(&diagnostics, &DIAG_OPTIONS),
            model_type: model_type.into(),
        }
    }

    /// Render a 3D model into a triangle mesh.
    #[wasm_bindgen(js_name = renderMesh)]
    pub fn render_mesh(&mut self) -> Result<Mesh, String> {
        Ok(TriangleMesh::from(&self.render()?.to_3d()).into())
    }

    /// Render a 2D model into polygons with holes.
    #[wasm_bindgen(js_name = renderPolygons)]
    pub fn render_polygons(&mut self) -> Result<Polygons, String> {
        Ok(MultiPolygon::from(self.render()?.to_2d()).into())
    }

    /// Render a 2D model into line segments, e.g. the outlines of polygons.
    #[wasm_bindgen(js_name = renderLines)]
    pub fn render_lines(&mut self) -> Result<Lines, String> {
        let mut lines = Lines::default();
        self.render()?
            .to_2d()
            .iter()
            .for_each(|geometry| lines.add(geometry));
        Ok(lines)
    }

    /// Render a 2D model into a triangle mesh in the XY plane.
    #[wasm_bindgen(js_name = renderTriangles)]
    pub fn render_triangles(&mut self) -> Result<Mesh, String> {
        Ok(Mesh::triangulate(&MultiPolygon::from(
            self.render()?.to_2d(),
        )))
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

/// The result of a compilation.
#[wasm_bindgen]
pub struct CompileResult {
    success: bool,
    diagnostics: String,
    model_type: String,
}

#[wasm_bindgen]
impl CompileResult {
    /// `true` if the code compiled without errors, so the model can be rendered and exported.
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Errors, warnings and advices as text.
    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> String {
        self.diagnostics.clone()
    }

    /// The type of the model: `"2d"`, `"3d"`, `"mixed"` or `"none"`.
    #[wasm_bindgen(getter, js_name = modelType)]
    pub fn model_type(&self) -> String {
        self.model_type.clone()
    }
}

/// A triangle mesh, e.g. for a `THREE.BufferGeometry`.
#[wasm_bindgen]
#[derive(Default)]
pub struct Mesh {
    positions: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl Mesh {
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
#[derive(Default)]
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
#[derive(Default)]
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
