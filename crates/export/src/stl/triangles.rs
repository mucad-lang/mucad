// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use cgmath::Vector3;
use mucad_core::{
    Geometries3D, Geometry, Geometry3D, Manifold, Triangle, TriangleMesh, UnaryBooleanOp,
};
use mucad_render::{GeometryNodeData, GeometryNodeRef, GeometryOutputs, GeometryTree};

pub type TriangleIter = Box<dyn Iterator<Item = Triangle<Vector3<f32>>>>;

pub trait AsTriangleIterator {
    fn triangle_iter(&self) -> TriangleIter;
}

impl AsTriangleIterator for TriangleMesh {
    fn triangle_iter(&self) -> TriangleIter {
        Box::new(
            self.triangles()
                .map(|tri| Triangle(*tri.0, *tri.1, *tri.2))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

impl AsTriangleIterator for Manifold {
    fn triangle_iter(&self) -> TriangleIter {
        let mesh: TriangleMesh = self.get_mesh_gl(0).into();
        mesh.triangle_iter()
    }
}

impl AsTriangleIterator for Geometry3D {
    fn triangle_iter(&self) -> TriangleIter {
        match self {
            Geometry3D::Mesh(mesh) => mesh.triangle_iter(),
            Geometry3D::Manifold(manifold) => manifold.triangle_iter(),
            Geometry3D::Collection(geometries) => geometries.triangle_iter(),
        }
    }
}

impl AsTriangleIterator for Geometries3D {
    fn triangle_iter(&self) -> TriangleIter {
        self.union().triangle_iter()
    }
}

impl AsTriangleIterator for Geometry {
    fn triangle_iter(&self) -> TriangleIter {
        match self {
            Self::Geometry3D(geo3d) => geo3d.triangle_iter(),
            Self::Geometry2D(_) => Box::new(std::iter::empty()),
        }
    }
}

impl AsTriangleIterator for GeometryOutputs {
    fn triangle_iter(&self) -> TriangleIter {
        let primary = self.clone().primary();
        primary.to_3d().union().triangle_iter()
    }
}

impl AsTriangleIterator for GeometryNodeData {
    fn triangle_iter(&self) -> TriangleIter {
        self.outputs.triangle_iter()
    }
}

impl AsTriangleIterator for GeometryTree {
    fn triangle_iter(&self) -> TriangleIter {
        fn recurse(node: GeometryNodeRef) -> Vec<Triangle<Vector3<f32>>> {
            if node.outputs.is_empty() {
                // Traverse into children until we have an output
                node.children().flat_map(recurse).collect()
            } else {
                node.outputs.triangle_iter().collect()
            }
        }

        Box::new(recurse(self.root()).into_iter())
    }
}
