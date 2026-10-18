// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Mat4, Scalar, Vec2};

#[derive(Debug, Clone)]
pub struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    pub fn new(min: impl Into<Vec2>, max: impl Into<Vec2>) -> Self {
        Self {
            min: min.into(),
            max: max.into(),
        }
    }

    pub fn translated(self, d: impl Into<Vec2>) -> Self {
        let d = d.into();
        Self::new(self.min + d, self.max + d)
    }

    pub fn center(&self) -> Vec2 {
        0.5 * (self.min + self.max)
    }

    pub fn width(&self) -> Scalar {
        (self.max.x - self.min.x).abs()
    }

    pub fn height(&self) -> Scalar {
        (self.max.y - self.min.y).abs()
    }

    pub fn aspect_ratio(&self) -> Scalar {
        self.width() / self.height()
    }
}

/// A zoom and pan model for 2D orthographic projections, typically used for
/// cameras or viewports in 2D or top-down 3D applications.
///
/// Maintains two `Rect` fields:
/// - `overview`: the full scene bounds (unchanging).
/// - `detail`: the current view (zoomed + panned).
///
/// Provides zooming and panning functionality while keeping the detail view
/// constrained within the overview bounds.
pub struct OrthoZoomModel {
    /// The full scene rectangle — the outer bounds for zooming and panning.
    overview: Rect,

    /// The current view rectangle (camera/viewport), potentially zoomed and panned.
    detail: Rect,

    /// Min and max zoom levels (clamps `set_zoom_level` and `zoom_to_point`).
    zoom_range: (Scalar, Scalar),
}

impl OrthoZoomModel {
    /// Creates a new `OrthoZoomModel` with a square scene radius and a target window size
    /// to compute aspect ratio.
    pub fn new(scene_radius: Scalar, window_size: (Scalar, Scalar)) -> Self {
        let r = scene_radius;
        let aspect = window_size.0 / window_size.1;

        Self {
            overview: Rect::new((-r * aspect, -r), (r * aspect, r)),
            detail: Rect::new((-r * aspect, -r), (r * aspect, r)),
            zoom_range: (0.01, 100.0), // Zoom from 1% to 10,000%
        }
    }

    /// Sets the overview rectangle (scene bounds).
    pub fn set_overview(&mut self, overview: Rect) {
        self.overview = overview;
    }

    /// Sets the current detail rectangle (viewport).
    pub fn set_detail(&mut self, detail: Rect) {
        self.detail = detail;
    }

    /// Translates (pans) the detail rectangle by a delta, constrained within overview bounds.
    pub fn translate(&mut self, mut d: Vec2) {
        // Clamp movement to prevent panning beyond overview bounds
        if (d.x < 0.0 && self.detail.min.x < self.overview.min.x)
            || (d.x > 0.0 && self.detail.max.x > self.overview.max.x)
        {
            d.x = 0.0;
        }

        if (d.y < 0.0 && self.detail.min.y < self.overview.max.y)
            || (d.y > 0.0 && self.detail.max.y < self.overview.max.y)
        {
            d.y = 0.0;
        }

        self.set_detail(self.detail.clone().translated(d))
    }

    /// Moves the viewport to center on the given point.
    pub fn move_to(&mut self, p: Vec2) {
        self.set_detail(self.detail.clone().translated(self.detail.center() - p))
    }

    /// Zooms the viewport in/out toward a given pivot point.
    ///
    /// The zoom factor should be:
    /// - `< 1.0` to zoom in
    /// - `> 1.0` to zoom out
    ///
    /// Clamped by `min_zoom_level` and `max_zoom_level`.
    pub fn zoom_to_point(&mut self, pivot: Vec2, zoom_factor: Scalar) {
        if zoom_factor < 1.0 && self.zoom_level() >= self.max_zoom_level() {
            return;
        }
        if zoom_factor > 1.0 && self.zoom_level() <= self.min_zoom_level() {
            return;
        }

        self.set_detail(zoomed_rect(self.detail.clone(), zoom_factor, pivot));
    }

    /// Returns the current zoom level (lower means more zoomed out).
    ///
    /// This is computed as the width difference between the overview and the detail.
    pub fn zoom_level(&self) -> Scalar {
        self.overview.width() - self.detail.width()
    }

    /// Returns the minimum allowed zoom level.
    pub fn min_zoom_level(&self) -> Scalar {
        self.zoom_range.0
    }

    /// Returns the maximum allowed zoom level.
    pub fn max_zoom_level(&self) -> Scalar {
        self.zoom_range.1
    }

    /// Sets the zoom level directly (clamped between min and max).
    ///
    /// Zoom level here is an abstract value — higher = more zoomed in.
    pub fn set_zoom_level(&mut self, mut zoom_level: Scalar) {
        zoom_level = zoom_level.clamp(self.zoom_range.0, self.zoom_range.1);

        let w = self.overview.width();
        let size = 0.5 * Vec2::new(w / zoom_level, w / self.aspect_ratio() / zoom_level);

        let c = self.detail.center();

        self.set_detail(Rect {
            min: c - size,
            max: c + size,
        });
    }

    /// Resets the zoom level to 1.0.
    pub fn reset(&mut self) {
        self.set_zoom_level(1.0)
    }

    /// Returns the aspect ratio of the current viewport (width / height).
    pub fn aspect_ratio(&self) -> Scalar {
        self.detail.aspect_ratio()
    }

    /// Returns the orthographic projection matrix for the current detail rectangle.
    ///
    /// Suitable for rendering the zoomed-in portion.
    pub fn detail_projection_matrix(&self) -> Mat4 {
        let r = &self.detail;
        cgmath::ortho(r.min.x, r.max.x, r.min.y, r.max.y, 0.1, 100000.0)
    }

    /// Returns the orthographic projection matrix for the overview rectangle.
    ///
    /// Suitable for rendering the entire scene without zoom.
    pub fn overview_projection_matrix(&self) -> Mat4 {
        let r = &self.overview;
        cgmath::ortho(r.min.x, r.max.x, r.min.y, r.max.y, 0.1, 100000.0)
    }

    /// Returns a "buffered" rectangle larger than the overview, useful for culling
    /// or visualizing out-of-bounds behavior.
    pub fn boundary_rect(&self) -> Rect {
        let s = Vec2::new(self.detail.width(), self.detail.height());

        Rect {
            min: self.overview.min - s,
            max: self.overview.max + s,
        }
    }
}

/// Returns a new rectangle zoomed around a pivot point.
///
/// The zoom works by scaling both corners relative to the pivot.
fn zoomed_rect(rect: Rect, zoom_factor: Scalar, pivot: Vec2) -> Rect {
    Rect::new(
        (rect.min - pivot) * zoom_factor,
        (rect.max - pivot) * zoom_factor,
    )
    .translated((pivot.x, pivot.y))
}
