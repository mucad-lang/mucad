// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render trait.

use cgmath::InnerSpace;

use microcad_core as core;

/// Render resolution when rendering things to polygons or meshes.
#[derive(Debug, Clone)]
pub struct RenderResolution {
    /// Linear resolution in millimeters (Default = 0.1mm)
    pub linear: core::Scalar,
}

impl RenderResolution {
    /// Create new render resolution.
    pub fn new(linear: core::Scalar) -> Self {
        Self { linear }
    }

    /// Coarse render resolution of 1.0mm.
    pub fn coarse() -> Self {
        Self { linear: 1.0 }
    }

    /// Medium render resolution of 0.25mm.
    pub fn medium() -> Self {
        Self { linear: 0.25 }
    }

    /// High render resolution of 0.1mm.
    pub fn high() -> Self {
        Self { linear: 0.1 }
    }

    /// Get the number segments for a circle as power of 2.
    ///
    /// The minimal number of segments is 4, the maximum number of segments is 1024.
    pub fn circular_segments(&self, radius: core::Scalar) -> u32 {
        let n = (radius / self.linear * std::f64::consts::PI * 0.5).max(3.0);
        2_u32.pow(n.log2().ceil() as u32).clamp(8, 1024)
    }
}

impl std::ops::Mul<core::Mat3> for RenderResolution {
    type Output = RenderResolution;

    fn mul(self, rhs: core::Mat3) -> Self::Output {
        let scale = (rhs.x.magnitude() * rhs.y.magnitude()).sqrt();
        Self {
            linear: self.linear / scale,
        }
    }
}

impl std::ops::Mul<core::Mat4> for RenderResolution {
    type Output = RenderResolution;

    fn mul(self, rhs: core::Mat4) -> Self::Output {
        let scale = (rhs.x.magnitude() * rhs.y.magnitude() * rhs.z.magnitude()).powf(1.0 / 3.0);
        Self {
            linear: self.linear / scale,
        }
    }
}

impl Default for RenderResolution {
    fn default() -> Self {
        RenderResolution::medium()
    }
}

impl std::fmt::Display for RenderResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}mm", self.linear)
    }
}
