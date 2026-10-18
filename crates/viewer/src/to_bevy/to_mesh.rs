// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Conversions from mucad types into bevy meshes.

use super::*;

use mucad_driver::prelude as mu;

/// Convert a mucad type into Bevy meshes with optional custom parameters.
///
/// A geometry collection results in several meshes.
pub trait ToBevyMesh {
    /// A custom parameter type to pass to the function.
    type Parameters: Default;

    /// The conversion function.
    fn to_bevy_mesh(&self, parameters: Self::Parameters) -> Vec<Mesh>;

    /// The conversion function with default parameters.
    fn to_bevy_mesh_default(&self) -> Vec<Mesh> {
        self.to_bevy_mesh(Self::Parameters::default())
    }
}

impl ToBevyMesh for mu::core::LineString {
    type Parameters = f32;

    fn to_bevy_mesh(&self, z: f32) -> Vec<Mesh> {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::LineStrip,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.coords()
                .map(|c| [c.x as f32, c.y as f32, z])
                .collect::<Vec<_>>(),
        );
        vec![mesh]
    }
}

impl ToBevyMesh for mu::core::MultiLineString {
    type Parameters = f32;

    fn to_bevy_mesh(&self, z: Self::Parameters) -> Vec<Mesh> {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::LineList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.0
                .iter()
                .flat_map(|line_string| {
                    line_string
                        .0
                        .as_slice()
                        .windows(2)
                        .flat_map(|c| c.iter().map(|c| [c.x as f32, c.y as f32, z]))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        );
        vec![mesh]
    }
}

#[derive(Default)]
struct Triangulation {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl Triangulation {
    /// Triangulate a single polygon.
    fn from_polygon(polygon: &mu::core::Polygon, z: f32, index_offset: usize) -> Self {
        use geo::TriangulateEarcut;
        let triangulation = polygon.earcut_triangles_raw();
        Self {
            positions: triangulation
                .vertices
                .iter()
                .map(|chunk| [chunk[0] as f32, chunk[1] as f32, z])
                .collect(),
            indices: triangulation
                .triangle_indices
                .iter()
                .map(|i| (i + index_offset) as u32)
                .collect(),
        }
    }

    /// Triangulate a multi-polygon.
    fn from_multi_polygon(multi_polygon: &mu::core::MultiPolygon, z: f32) -> Self {
        let mut triangulation = Self::default();
        for polygon in &multi_polygon.0 {
            let mut t = Self::from_polygon(polygon, z, triangulation.positions.len());
            triangulation.positions.append(&mut t.positions);
            triangulation.indices.append(&mut t.indices);
        }

        triangulation
    }

    /// Return triangulation as bevy mesh.
    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

impl ToBevyMesh for mu::core::Polygon {
    type Parameters = f32;

    fn to_bevy_mesh(&self, z: Self::Parameters) -> Vec<Mesh> {
        vec![Triangulation::from_polygon(self, z, 0).mesh()]
    }
}

impl ToBevyMesh for mu::core::MultiPolygon {
    type Parameters = f32;

    fn to_bevy_mesh(&self, z: Self::Parameters) -> Vec<Mesh> {
        vec![Triangulation::from_multi_polygon(self, z).mesh()]
    }
}

impl ToBevyMesh for mu::core::Geometry2D {
    type Parameters = f32;
    fn to_bevy_mesh(&self, z: Self::Parameters) -> Vec<Mesh> {
        use mu::core::Geometry2D;
        match self {
            Geometry2D::LineString(line_string) => line_string.to_bevy_mesh(z),
            Geometry2D::MultiLineString(multi_line_string) => multi_line_string.to_bevy_mesh(z),
            Geometry2D::Line(line) => {
                mu::core::LineString::new(vec![line.0.into(), line.1.into()]).to_bevy_mesh(z)
            }
            Geometry2D::Polygon(polygon) => polygon.to_bevy_mesh(z),
            Geometry2D::MultiPolygon(multi_polygon) => multi_polygon.to_bevy_mesh(z),
            Geometry2D::Collection(collection) => collection
                .iter()
                .flat_map(|geometry| geometry.to_bevy_mesh(z))
                .collect(),
        }
    }
}

/// Converts a TriangleMesh into a Bevy Mesh with smooth normals (within angle threshold),
/// and sharp edges where the angle between adjacent faces exceeds the threshold.
impl ToBevyMesh for mu::core::TriangleMesh {
    type Parameters = f32;

    fn to_bevy_mesh(&self, threshold_degrees: Self::Parameters) -> Vec<Mesh> {
        let cos_threshold = threshold_degrees.to_radians().cos();
        use cgmath::InnerSpace;

        let mut face_normals = Vec::with_capacity(self.triangle_indices.len());
        let mut vertex_to_faces: Vec<Vec<usize>> = vec![vec![]; self.positions.len()];
        for (face_idx, mu::core::Triangle(i0, i1, i2)) in self.triangle_indices.iter().enumerate() {
            let a = self.positions[*i0 as usize];
            let b = self.positions[*i1 as usize];
            let c = self.positions[*i2 as usize];

            let normal = (b - a).cross(c - a).normalize();
            face_normals.push(normal);

            for &i in [i0, i1, i2] {
                vertex_to_faces[i as usize].push(face_idx);
            }
        }

        #[derive(Hash, Eq, PartialEq, Debug)]
        struct VertexKey {
            index: usize,
            quant_normal: [i16; 3], // Normal, quantized
        }

        // Quantize normal into hashable form
        fn quantize_normal(n: Vec3) -> [i16; 3] {
            [
                (n.x.clamp(-1.0, 1.0) * 32767.0) as i16,
                (n.y.clamp(-1.0, 1.0) * 32767.0) as i16,
                (n.z.clamp(-1.0, 1.0) * 32767.0) as i16,
            ]
        }

        let mut vertex_cache: HashMap<VertexKey, u32> = HashMap::new();
        let mut new_positions = Vec::new();
        let mut new_normals = Vec::new();
        let mut new_indices = Vec::new();

        for (face_idx, mu::core::Triangle(i0, i1, i2)) in self.triangle_indices.iter().enumerate() {
            for &orig_idx in [i0, i1, i2] {
                let pos = self.positions[orig_idx as usize];

                // Average normals of adjacent faces within the angle threshold
                let mut normal_sum = Vec3::ZERO;
                for &adj_face_idx in &vertex_to_faces[orig_idx as usize] {
                    let adj_normal = face_normals[adj_face_idx];
                    if face_normals[face_idx].dot(adj_normal) >= cos_threshold {
                        normal_sum += Vec3::new(adj_normal.x, adj_normal.y, adj_normal.z);
                    }
                }

                let smooth_normal = normal_sum.normalize_or_zero();
                let key = VertexKey {
                    index: orig_idx as usize,
                    quant_normal: quantize_normal(smooth_normal),
                };

                new_indices.push(match vertex_cache.get(&key) {
                    Some(idx) => *idx,
                    None => {
                        let idx = new_positions.len() as u32;
                        new_positions.push([pos.x, pos.y, pos.z]);
                        new_normals.push(smooth_normal);
                        vertex_cache.insert(key, idx);
                        idx
                    }
                });
            }
        }

        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
        mesh.insert_indices(Indices::U32(new_indices));
        vec![mesh]
    }
}

/// Create a bevy mesh from a 3D geometry.
impl ToBevyMesh for mu::core::Geometry3D {
    type Parameters = f32;

    fn to_bevy_mesh(&self, threshold_angle: Self::Parameters) -> Vec<Mesh> {
        use mu::core::Geometry3D;
        match self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.to_bevy_mesh(threshold_angle),
            Geometry3D::Manifold(manifold) => {
                mu::core::TriangleMesh::from(manifold.get_mesh_gl(0)).to_bevy_mesh(threshold_angle)
            }
            Geometry3D::Collection(collection) => {
                let mesh: mu::core::TriangleMesh = collection.into();
                mesh.to_bevy_mesh(threshold_angle)
            }
        }
    }
}

/// Create mesh from a [`Bounds2D`].
impl ToBevyMesh for mu::core::Bounds2D {
    type Parameters = ();

    fn to_bevy_mesh(&self, _: Self::Parameters) -> Vec<Mesh> {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::LineStrip,
            RenderAssetUsages::default(),
        );
        use bevy::prelude::{Vec2, Vec3};
        let min = Vec2::new(self.min.x as f32, self.min.y as f32);
        let max = Vec2::new(self.max.x as f32, self.max.y as f32);
        let z = 0.0_f32;
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            [
                Vec3::new(min.x, min.y, z),
                Vec3::new(max.x, min.y, z),
                Vec3::new(max.x, min.y, z),
                Vec3::new(min.x, min.y, z),
                Vec3::new(min.x, min.y, z),
            ]
            .iter()
            .map(|p| [p.x, p.y, p.z])
            .collect::<Vec<_>>(),
        );
        vec![mesh]
    }
}

/// Create mesh from a [`Bounds3D`].
impl ToBevyMesh for mu::core::Bounds3D {
    type Parameters = ();

    fn to_bevy_mesh(&self, _: Self::Parameters) -> Vec<Mesh> {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::LineStrip,
            RenderAssetUsages::default(),
        );
        use bevy::prelude::Vec3;
        let min = Vec3::new(self.min.x as f32, self.min.y as f32, self.min.z as f32);
        let max = Vec3::new(self.max.x as f32, self.max.y as f32, self.max.z as f32);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            [
                // Bottom face
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, min.y, max.z),
                Vec3::new(min.x, min.y, max.z),
                Vec3::new(min.x, min.y, min.z), // close bottom loop
                // Connect to top face
                Vec3::new(min.x, max.y, min.z),
                // Top face
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(min.x, max.y, max.z),
                Vec3::new(min.x, max.y, min.z), // close top loop
                // Back down to start
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, min.y, max.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(min.x, max.y, max.z),
                Vec3::new(min.x, min.y, max.z),
            ]
            .iter()
            .map(|p| [p.x, p.y, p.z])
            .collect::<Vec<_>>(),
        );
        vec![mesh]
    }
}
