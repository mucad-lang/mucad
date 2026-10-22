// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generic triangle.

/// Triangle
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Triangle<T>(pub T, pub T, pub T);

// Indexed triangles can be viewed as flat indices, see `TriangleMesh::indices`.
unsafe impl bytemuck::Zeroable for Triangle<u32> {}
unsafe impl bytemuck::Pod for Triangle<u32> {}

/// Implementation for indexed triangle.
impl Triangle<u32> {
    /// A triangle is generated if it contains any repeated index.
    pub fn is_degenerated(&self) -> bool {
        self.0 == self.1 || self.1 == self.2 || self.2 == self.0
    }

    /// Returns a flipped version of this triangle, by swapping first and third index.
    pub fn flipped(self) -> Triangle<u32> {
        Triangle(self.2, self.1, self.0)
    }
}
