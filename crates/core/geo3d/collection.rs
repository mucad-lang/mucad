// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! 3D Geometry collection

use derive_more::{Deref, DerefMut};

use crate::{
    geo3d::{CalcBounds3D, bounds::Bounds3D},
    traits::{DistributeGrid, TotalMemory, TransformAffine, VertexCount},
    *,
};

/// 3D geometry collection.
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct Geometries3D(Vec<Geometry3D>);

impl Geometries3D {
    /// New geometry collection.
    pub fn new(geometries: Vec<Geometry3D>) -> Self {
        Self(geometries.into_iter().collect())
    }

    /// Append another geometry collection.
    pub fn append(&mut self, mut geometries: Geometries3D) {
        self.0.append(&mut geometries.0)
    }
}

impl UnaryBooleanOp for Geometries3D {
    type Output = Manifold;

    fn boolean_op(&self, op: BooleanOp) -> Self::Output {
        let manifold_list: Vec<_> = self
            .0
            .iter()
            // Render each geometry into a multipolygon and filter out empty ones
            .filter_map(|geo| {
                let manifold: Manifold = geo.clone().into();
                if manifold.is_empty() {
                    None
                } else {
                    Some(manifold)
                }
            })
            .collect();

        if manifold_list.is_empty() {
            return Manifold::empty();
        }

        let op = op.into();
        manifold_list[1..]
            .iter()
            .fold(manifold_list[0].clone(), |acc, other| {
                acc.boolean(other, op)
            })
    }
}

impl FromIterator<Geometry3D> for Geometries3D {
    fn from_iter<T: IntoIterator<Item = Geometry3D>>(iter: T) -> Self {
        Geometries3D(iter.into_iter().collect())
    }
}

impl CalcBounds3D for Geometries3D {
    fn calc_bounds_3d(&self) -> Bounds3D {
        self.0.iter().fold(Bounds3D::default(), |bounds, geometry| {
            bounds.extend(geometry.calc_bounds_3d())
        })
    }
}

impl TransformAffine for Geometries3D {
    fn transform_affine(&mut self, m: &Mat4) {
        for geometry in &mut self.0 {
            geometry.transform_affine(m);
        }
    }
}

impl Transformed3D for Geometries3D {
    fn transformed_3d(&self, mat: &Mat4) -> Self {
        Self(
            self.iter()
                .map(|geometry| geometry.transformed_3d(mat))
                .collect::<Vec<_>>(),
        )
    }
}

impl DistributeGrid for Geometries3D {
    fn distribute_grid(&self, rect: Rect, rows: Integer, columns: Integer) -> Self {
        Geometries3D(
            GridCells::new(rect, rows, columns)
                .zip(self.0.iter())
                .map(|(cell, geo)| {
                    let bounds = geo.calc_bounds_3d();
                    let center = bounds.center();
                    let cell_center: Vec2 = cell.center().x_y().into();
                    let d = center - cell_center.extend(bounds.min.z + center.z);
                    geo.transformed_3d(&Mat4::from_translation(d))
                })
                .collect(),
        )
    }
}

impl TotalMemory for Geometries3D {
    fn heap_memory(&self) -> usize {
        self.iter().map(|g| g.heap_memory()).sum()
    }
}

impl VertexCount for Geometries3D {
    fn vertex_count(&self) -> usize {
        self.iter().map(|g| g.vertex_count()).sum()
    }
}
