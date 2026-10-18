// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render attributes.

use mucad_core::{Color, Scalar};

/// An attribute that can be used by any renderer.
///
/// *Note: Render color is the only supported attribute for now.*
#[non_exhaustive]
#[derive(Clone, Default, Debug)]
pub struct RenderAttributes {
    /// Fill color,  e.g. used when rendering SVGs.
    pub fill_color: Option<Color>,
    /// Stroke color, e.g. used when rendering SVGs.
    pub stroke_color: Option<Color>,
    /// Stroke with, e.g. used when rendering SVGs.
    pub stroke_width: Option<Scalar>,
}

impl RenderAttributes {
    /// Helper to set both fill and stroke to the same color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self.stroke_color = Some(color);
        self
    }
}
