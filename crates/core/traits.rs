// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad core geometry traits

use crate::{Integer, Mat4, Rect};

/// Trait to align something to center.
pub trait Center<T = Self> {
    /// Align geometry.
    fn center(&self) -> T;
}

/// Trait for geometry types that can be transformed by a 4x4 affine matrix.
pub trait TransformAffine {
    /// Transforms `self` in place using an affine transformation matrix.
    fn transform_affine(&mut self, m: &Mat4);

    /// Consumes `self` and returns the transformed value.
    ///
    /// Has a default implementation that mutates `self` in place.
    #[must_use]
    fn into_affine_transformed(mut self, m: &Mat4) -> Self
    where
        Self: Sized,
    {
        self.transform_affine(m);
        self
    }
}

/// Trait to distribute geometries in a 2D grid.
pub trait DistributeGrid<T = Self> {
    /// Distribute in a grid.
    fn distribute_grid(&self, rect: Rect, rows: Integer, columns: Integer) -> T;
}

/// Return total amount of memory in bytes.
pub trait TotalMemory {
    /// Total amount of memory in bytes.
    fn total_memory(&self) -> usize {
        self.stack_memory() + self.heap_memory()
    }

    /// Get amount of stack memory in bytes.
    fn stack_memory(&self) -> usize {
        std::mem::size_of_val(self)
    }

    /// Get amount of heap memory in bytes.
    fn heap_memory(&self) -> usize {
        0
    }
}

impl<T> TotalMemory for Vec<T> {
    fn heap_memory(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
    }
}

/// Return number of vertices.
pub trait VertexCount {
    /// Return vertex count.
    fn vertex_count(&self) -> usize;
}
