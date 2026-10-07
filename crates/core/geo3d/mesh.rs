// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    traits::{TotalMemory, TransformAffine, VertexCount},
    *,
};
use cgmath::{ElementWise, Vector3};
use manifold_rust::{manifold::Manifold, types::MeshGL};

use microcad_hash::HashMap;

/// Triangle mesh
#[derive(Default, Clone)]
pub struct TriangleMesh {
    /// Mesh Vertices
    pub positions: Vec<Vector3<f32>>,
    /// Optional normals.
    pub normals: Option<Vec<Vector3<f32>>>,
    /// Triangle indices.
    pub triangle_indices: Vec<Triangle<u32>>,
}
/// Triangle iterator state.
pub struct Triangles<'a> {
    triangle_mesh: &'a TriangleMesh,
    index: usize,
}

impl<'a> Iterator for Triangles<'a> {
    type Item = Triangle<&'a Vector3<f32>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.triangle_mesh.triangle_indices.len() {
            let t = self.triangle_mesh.triangle_indices[self.index];
            self.index += 1;
            Some(Triangle(
                &self.triangle_mesh.positions[t.0 as usize],
                &self.triangle_mesh.positions[t.1 as usize],
                &self.triangle_mesh.positions[t.2 as usize],
            ))
        } else {
            None
        }
    }
}

impl TriangleMesh {
    /// Is this mesh empty?
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty() || self.triangle_indices.is_empty()
    }

    /// Clear mesh.
    pub fn clear(&mut self) {
        self.positions.clear();
        self.triangle_indices.clear();
    }

    /// Fetch triangles.
    pub fn fetch_triangles(&self) -> Vec<Triangle<Vector3<f32>>> {
        self.triangle_indices
            .iter()
            .map(|t| {
                Triangle(
                    self.positions[t.0 as usize],
                    self.positions[t.1 as usize],
                    self.positions[t.2 as usize],
                )
            })
            .collect()
    }

    /// Append a triangle mesh.
    pub fn append(&mut self, other: &TriangleMesh) {
        let offset = self.positions.len() as u32;
        self.positions.append(&mut other.positions.clone());
        self.triangle_indices.extend(
            other
                .triangle_indices
                .iter()
                .map(|t| Triangle(t.0 + offset, t.1 + offset, t.2 + offset)),
        )
    }

    /// Triangles iterator.
    pub fn triangles(&'_ self) -> Triangles<'_> {
        Triangles {
            triangle_mesh: self,
            index: 0,
        }
    }

    /// Convert mesh to manifold.
    pub fn to_manifold(&self) -> Manifold {
        let vertices = self
            .positions
            .iter()
            .flat_map(|v| [v.x, v.y, v.z])
            .collect::<Vec<_>>();

        let triangle_indices = self
            .triangle_indices
            .iter()
            .flat_map(|t| [t.0, t.1, t.2])
            .collect::<Vec<_>>();

        assert_eq!(vertices.len(), self.positions.len() * 3);
        assert_eq!(triangle_indices.len(), self.triangle_indices.len() * 3);

        let mesh = MeshGL::from(self.clone());

        Manifold::from_mesh_gl(&mesh)
    }

    /// Calculate volume of mesh.
    pub fn volume(&self) -> f64 {
        self.triangles()
            .map(|t| t.signed_volume() as f64)
            .sum::<f64>()
            .abs()
    }

    /// Fetch a vertex triangle from index triangle.
    pub fn fetch_triangle(&self, tri: Triangle<u32>) -> Triangle<&Vector3<f32>> {
        Triangle(
            &self.positions[tri.0 as usize],
            &self.positions[tri.1 as usize],
            &self.positions[tri.2 as usize],
        )
    }

    /// TriangleMesh.
    pub fn repair(&mut self, bounds: &Bounds3D) {
        // 1. Merge duplicate vertices using a spatial hash map (or hashmap keyed on quantized position)

        let min: Vector3<f32> = bounds.min.cast().expect("Successful cast");
        let inv_size: Vector3<f32> = (1.0 / (bounds.max - bounds.min))
            .cast()
            .expect("Successful cast");

        // Quantize vertex positions to grid to group duplicates
        let quantize = |pos: &Vector3<f32>| {
            let mapped = (pos - min).mul_element_wise(inv_size) * (u32::MAX as f32);
            (
                mapped.x.floor() as u32,
                mapped.y.floor() as u32,
                mapped.z.floor() as u32,
            )
        };

        let mut vertex_map: HashMap<(u32, u32, u32), u32> = HashMap::default();
        let mut new_positions: Vec<Vector3<f32>> = Vec::with_capacity(self.positions.len());
        let remap: Vec<u32> = self
            .positions
            .iter()
            .map(|position| {
                let key = quantize(position);
                if let Some(&existing_idx) = vertex_map.get(&key) {
                    // Duplicate vertex found
                    existing_idx
                } else {
                    // New unique vertex
                    let new_idx = new_positions.len() as u32;
                    new_positions.push(*position);
                    vertex_map.insert(key, new_idx);
                    new_idx
                }
            })
            .collect();

        self.positions = new_positions;

        // 2. Remap triangle indices and remove degenerate triangles (zero area or repeated vertices)
        let mut new_triangles = Vec::with_capacity(self.triangle_indices.len());

        for tri in &self.triangle_indices {
            let tri_idx = crate::Triangle(
                remap[tri.0 as usize],
                remap[tri.1 as usize],
                remap[tri.2 as usize],
            );

            if tri_idx.is_degenerated() {
                continue;
            }

            // Optional: check zero-area triangle by computing cross product
            let tri = self.fetch_triangle(tri_idx);

            if tri.area() < 1e-8 {
                continue; // Degenerate triangle
            }

            new_triangles.push(tri_idx);
        }

        self.triangle_indices = new_triangles;
    }
}

impl CalcBounds3D for TriangleMesh {
    fn calc_bounds_3d(&self) -> Bounds3D {
        self.positions
            .iter()
            .map(|positions| positions.cast::<f64>().expect("Successful cast"))
            .collect()
    }
}

impl From<MeshGL> for TriangleMesh {
    fn from(mesh: MeshGL) -> Self {
        let vertices = mesh.vert_properties;
        let indices = mesh.tri_verts;

        // TODO: We could use unsafe std::ptr::copy and cast::transmute to avoid deep copy
        // of vertices and indices

        TriangleMesh {
            positions: (0..vertices.len())
                .step_by(3)
                .map(|i| Vector3::new(vertices[i], vertices[i + 1], vertices[i + 2]))
                .collect(),
            normals: None,
            triangle_indices: (0..indices.len())
                .step_by(3)
                .map(|i| Triangle(indices[i], indices[i + 1], indices[i + 2]))
                .collect(),
        }
    }
}

impl From<TriangleMesh> for MeshGL {
    fn from(mesh: TriangleMesh) -> Self {
        let vert_properties = mesh
            .positions
            .iter()
            .flat_map(|v| [v.x, v.y, v.z])
            .collect::<Vec<_>>();

        let tri_verts = mesh
            .triangle_indices
            .iter()
            .flat_map(|t| [t.0, t.1, t.2])
            .collect::<Vec<_>>();

        assert_eq!(vert_properties.len(), mesh.positions.len() * 3);
        assert_eq!(tri_verts.len(), mesh.triangle_indices.len() * 3);

        Self {
            num_prop: 3,
            vert_properties,
            tri_verts,
            ..Default::default()
        }
    }
}

impl From<Manifold> for TriangleMesh {
    fn from(manifold: Manifold) -> Self {
        TriangleMesh::from(manifold.get_mesh_gl(0))
    }
}

impl Transformed3D for TriangleMesh {
    fn transformed_3d(&self, mat: &Mat4) -> Self {
        let mat = mat.cast::<f32>().expect("Successful cast");
        let normals = match &self.normals {
            Some(normals) => {
                let rot_mat = cgmath::Matrix3::from_cols(
                    mat.x.truncate(),
                    mat.y.truncate(),
                    mat.z.truncate(),
                );
                let normals = normals.iter().map(|n| rot_mat * n).collect();
                Some(normals)
            }
            None => None,
        };

        Self {
            positions: self
                .positions
                .iter()
                .map(|v| (mat * v.extend(1.0)).truncate())
                .collect(),
            normals,
            triangle_indices: self.triangle_indices.clone(),
        }
    }
}

impl WithBounds3D<TriangleMesh> {
    /// Update bounds and repair mesh.
    pub fn repair(&mut self) {
        self.update_bounds();
        self.inner.repair(&self.bounds);
    }
}

impl TotalMemory for TriangleMesh {
    fn heap_memory(&self) -> usize {
        self.positions.heap_memory()
            + self.triangle_indices.heap_memory()
            + match &self.normals {
                Some(normals) => normals.heap_memory(),
                None => 0,
            }
    }
}

impl VertexCount for TriangleMesh {
    fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

use cgmath::{InnerSpace, Vector4};

impl TransformAffine for TriangleMesh {
    fn transform_affine(&mut self, m: &Mat4) {
        // 1. Transform positions (points have w = 1.0)
        for pos in &mut self.positions {
            // Convert Vector3<f32> to f64 for matrix multiplication, then back to f32
            let p4 = Vector4::new(pos.x as f64, pos.y as f64, pos.z as f64, 1.0);
            let transformed = m * p4;

            // Handle potential perspective divide if matrix is non-affine,
            // though standard affine matrices will have w == 1.0.
            let w = if transformed.w != 0.0 {
                transformed.w
            } else {
                1.0
            };
            *pos = Vector3::new(
                (transformed.x / w) as f32,
                (transformed.y / w) as f32,
                (transformed.z / w) as f32,
            );
        }

        // 2. Transform normals (directional vectors have w = 0.0)
        if let Some(normals) = &mut self.normals {
            for norm in normals {
                let n4 = Vector4::new(norm.x as f64, norm.y as f64, norm.z as f64, 0.0);
                let transformed = (m * n4).truncate(); // drop w

                let transformed_f32 = Vector3::new(
                    transformed.x as f32,
                    transformed.y as f32,
                    transformed.z as f32,
                );

                // Renormalize to maintain unit length after scale/rotation
                *norm = transformed_f32.normalize();
            }
        }

        // Triangle indices remain unchanged as topology is invariant under affine transform.
    }
}

impl From<Geometry3D> for TriangleMesh {
    fn from(geo: Geometry3D) -> Self {
        match geo {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh,
            Geometry3D::Manifold(manifold) => manifold.get_mesh_gl(0).into(),
            Geometry3D::Collection(ref collection) => collection.into(),
        }
    }
}

impl From<&Geometry3D> for TriangleMesh {
    fn from(geo: &Geometry3D) -> Self {
        match geo {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.clone(),
            Geometry3D::Manifold(manifold) => manifold.get_mesh_gl(0).into(),
            Geometry3D::Collection(collection) => collection.into(),
        }
    }
}

impl From<&Geometries3D> for TriangleMesh {
    fn from(geo: &Geometries3D) -> Self {
        geo.boolean_op(BooleanOp::Union).get_mesh_gl(0).into()
    }
}

#[test]
fn test_triangle_mesh_transform() {
    let mesh = TriangleMesh {
        positions: vec![
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(1.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
        ],
        normals: None,
        triangle_indices: vec![Triangle(0, 1, 2)],
    };

    let mesh = mesh.transformed_3d(&crate::Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));

    assert_eq!(mesh.positions[0], cgmath::Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(mesh.positions[1], cgmath::Vector3::new(2.0, 2.0, 3.0));
    assert_eq!(mesh.positions[2], cgmath::Vector3::new(1.0, 3.0, 3.0));
}
