// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::Scalar;

/// 2D size in millimeters.
#[derive(Clone, Default, Debug)]
pub struct Size2 {
    /// Width in mm.
    pub width: Scalar,
    /// Height in mm.
    pub height: Scalar,
}

impl Size2 {
    /// A4 sheet.
    pub const A4: Size2 = Size2 {
        width: 210.0,
        height: 297.0,
    };

    /// Create a new 2D size.
    pub fn new(width: Scalar, height: Scalar) -> Self {
        Self { width, height }
    }

    /// Calculate transposed version of this size.
    pub fn transposed(self) -> Self {
        Self::new(self.height, self.width)
    }
}

impl std::fmt::Display for Size2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Size2({} mm x {} mm)", self.width, self.height)
    }
}
