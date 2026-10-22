// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::rc::Rc;

use wasm_bindgen::prelude::wasm_bindgen;

use mucad_driver::prelude as mu;

/// A triangle mesh, e.g. for a `THREE.BufferGeometry`.
///
/// Clones share the mesh, e.g. the triangles of a sketch.
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Mesh {
    mesh: Rc<mu::core::TriangleMesh>,
    edges: Vec<u32>,
    color: Option<[f32; 4]>,
}

/// Builder methods
impl Mesh {
    pub fn with_color(mut self, color: impl Into<[f32; 4]>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// The triangle mesh, e.g. to test it natively, where the JavaScript arrays are not available.
    pub fn triangle_mesh(&self) -> &mu::core::TriangleMesh {
        &self.mesh
    }
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
    pub fn positions(&self) -> js_sys::Float32Array {
        js_sys::Float32Array::from(self.mesh.coordinates())
    }

    /// The vertex indices of the triangles as triples.
    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> js_sys::Uint32Array {
        js_sys::Uint32Array::from(self.mesh.indices())
    }

    /// The contour edges as pairs of vertex indices, e.g. for a `THREE.LineSegments` with the
    /// `positions` of this mesh.
    ///
    /// An edge is a contour, if the normals of its two triangles differ more than
    /// `threshold_angle` (in radians), or if it has not exactly two triangles, e.g. at an open
    /// border.
    pub fn edges(&self) -> js_sys::Uint32Array {
        js_sys::Uint32Array::from(self.edges.as_slice())
    }
}

impl Mesh {
    /// Triangulate polygons into a mesh in the XY plane.
    fn triangulate(multi_polygon: &mu::core::MultiPolygon) -> Self {
        use geo::TriangulateEarcut;

        let mut mesh = mu::core::TriangleMesh::default();
        for polygon in multi_polygon {
            let triangulation = polygon.earcut_triangles_raw();
            let offset = mesh.positions.len() as u32;
            mesh.positions.extend(
                triangulation
                    .vertices
                    .iter()
                    .map(|[x, y]| cgmath::Vector3::new(*x as f32, *y as f32, 0.0)),
            );
            mesh.triangle_indices.extend(
                triangulation
                    .triangle_indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|[a, b, c]| {
                        mu::core::Triangle(
                            offset + *a as u32,
                            offset + *b as u32,
                            offset + *c as u32,
                        )
                    }),
            );
        }
        mesh.into()
    }
}

impl From<mu::core::TriangleMesh> for Mesh {
    fn from(mesh: mu::core::TriangleMesh) -> Self {
        let edges = mesh.edges(cgmath::Rad(20.0.into())).into_flattened();
        Self {
            mesh: Rc::new(mesh),
            edges,
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
    pub fn new(color: Option<mu::Color>, outputs: &mu::render::GeometryOutputs) -> Self {
        let geometries = outputs.to_2d();
        let mut lines = Lines::default();
        geometries.iter().for_each(|geometry| lines.add(geometry));
        let multi_polygon = mu::core::MultiPolygon::from(geometries);
        Self {
            triangles: Mesh::triangulate(&multi_polygon),
            polygons: multi_polygon.into(),
            lines,
            color: color.map(|c| c.into()),
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

impl From<mu::core::MultiPolygon> for Polygons {
    fn from(multi_polygon: mu::core::MultiPolygon) -> Self {
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
    fn add(&mut self, geometry: &mu::core::Geometry2D) {
        use mu::core::Geometry2D::*;
        match geometry {
            LineString(line_string) => self.add_line_string(line_string),
            MultiLineString(multi_line_string) => multi_line_string
                .iter()
                .for_each(|line_string| self.add_line_string(line_string)),
            Line(line) => self.positions.extend([
                line.0.x() as f32,
                line.0.y() as f32,
                line.1.x() as f32,
                line.1.y() as f32,
            ]),
            Polygon(polygon) => self.add_polygon(polygon),
            MultiPolygon(multi_polygon) => multi_polygon
                .iter()
                .for_each(|polygon| self.add_polygon(polygon)),
            Collection(geometries) => geometries.iter().for_each(|geometry| self.add(geometry)),
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

/// The outputs grouped by their color, in the order of the first output of each color.
pub fn by_color(
    outputs: mu::render::GeometryOutputs,
) -> Vec<(Option<mu::Color>, mu::render::GeometryOutputs)> {
    let mut groups: Vec<(
        Option<mu::Color>,
        Vec<std::sync::Arc<mu::render::GeometryOutput>>,
    )> = Vec::new();
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
        .map(|(color, group)| (color, mu::render::GeometryOutputs::from_iter(group)))
        .collect()
}
