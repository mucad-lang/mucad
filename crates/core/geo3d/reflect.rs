// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reflect and mirror 3D geometries.
//!
//! `Mirror` duplicates and keep the original geometry, where is `Reflect` only transform the original geometry.

use super::*;
use cgmath::InnerSpace;

/// Reflects a 2D geometry along a line.
pub trait Reflect3D<T = Self> {
    /// Mirror a 2D geometry.
    fn reflect_3d(&self, plane: &Plane) -> T;
}

/// Mirrors a 2D geometry, keeping the original (in contrast to `reflect`).
pub trait Mirror3D<T = Self>: Reflect3D<T> + Into<Geometry3D> {
    /// Mirror operation.
    fn mirror_3d(self, plane: &Plane) -> Geometries3D {
        let orig: Geometry3D = self.into();
        let refl: Geometry3D = orig.reflect_3d(plane);
        Geometries3D::new(vec![orig, refl])
    }
}

impl Reflect3D for crate::Vec3 {
    fn reflect_3d(&self, plane: &Plane) -> Self {
        let n = plane.n.normalize(); // Ensure the normal is unit length
        let v = *self - plane.p; // Vector from plane point to the point
        let dist = v.dot(n); // Signed distance from plane
        *self - 2.0 * dist * n // Reflect across the plane
    }
}

impl Reflect3D for cgmath::Vector3<f32> {
    fn reflect_3d(&self, plane: &Plane) -> Self {
        let n: cgmath::Vector3<f32> = plane.n.normalize().cast().expect("Valid cast"); // Ensure the normal is unit length
        let p: cgmath::Vector3<f32> = plane.p.cast().expect("Valid cast");
        let v = *self - p; // Vector from plane point to the point
        let dist = v.dot(n); // Signed distance from plane
        *self - 2.0 * dist * n // Reflect across the plane
    }
}

impl Reflect3D for TriangleMesh {
    fn reflect_3d(&self, plane: &Plane) -> Self {
        Self {
            positions: self
                .positions
                .iter()
                .map(|pos| pos.reflect_3d(plane))
                .collect(),
            normals: self.normals.clone(), // Flip normals here?
            triangle_indices: self
                .triangle_indices
                .iter()
                .map(|tri| tri.flipped())
                .collect(),
        }
    }
}

impl Mirror3D for TriangleMesh {}

impl Reflect3D for Geometries3D {
    fn reflect_3d(&self, plane: &Plane) -> Self {
        Self::from_iter(self.iter().map(|geometry| geometry.reflect_3d(plane)))
    }
}

impl Mirror3D for Geometries3D {}

impl Reflect3D for Geometry3D {
    fn reflect_3d(&self, plane: &Plane) -> Self {
        match &self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.reflect_3d(plane).into(),
            Geometry3D::Manifold(manifold) => TriangleMesh::from(manifold.get_mesh_gl(0))
                .reflect_3d(plane)
                .into(),
            Geometry3D::Collection(collection) => collection.reflect_3d(plane).into(),
        }
    }
}

impl Mirror3D for Geometry3D {}
