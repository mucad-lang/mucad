// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canvas to draw geometry.

use geo::{Coord, MultiPolygon};
use microcad_core::{
    Bounds2D, Circle, Geometries2D, Geometry, Geometry2D, Line, LineString, MultiLineString, Point,
    Polygon, Rect, Scalar, Size2, Vec2, geo2d,
};
use microcad_render::{
    GeometryNodeData, GeometryNodeMut, GeometryOutput, GeometryOutputs, GeometryTree,
};

use crate::svg::CenteredText;

/// A canvas coordinate system.
#[derive(Clone, Debug)]
pub struct Canvas {
    /// The canvas rect.
    pub rect: geo2d::Rect,
    /// The content rect.
    pub content_rect: geo2d::Rect,
    /// Size2.
    pub size: Size2,
}

impl Canvas {
    /// Create a new canvas with a content rect.
    pub fn new(content_rect: geo2d::Rect) -> Self {
        let size = Size2 {
            width: content_rect.width(),
            height: content_rect.height(),
        };

        // Compute scale to fit content inside canvas (preserving aspect ratio)
        let scale = {
            let scale_x = size.width / content_rect.width();
            let scale_y = size.height / content_rect.height();
            scale_x.min(scale_y)
        };

        // New content size after scaling
        let width = content_rect.width() * scale;
        let height = content_rect.height() * scale;

        // Center the content within the canvas
        let min = Point::new((size.width - width) / 2.0, (size.height - height) / 2.0);

        // Build the new canvas rect centered with content
        let rect = geo2d::Rect::new(min, min + geo2d::Point::new(width, height));

        Canvas {
            rect,
            content_rect,
            size,
        }
    }

    /// Create a new canvas with a size and center the content.
    pub fn new_centered(
        content_rect: geo2d::Rect,
        size: Option<Size2>,
        scale: Option<Scalar>,
    ) -> Self {
        let size = size.unwrap_or_else(|| Size2 {
            width: content_rect.width(),
            height: content_rect.height(),
        });

        // Compute scale to fit content inside canvas (preserving aspect ratio)
        let scale = match scale {
            Some(scale) => scale,
            None => {
                let scale_x = size.width / content_rect.width();
                let scale_y = size.height / content_rect.height();
                scale_x.min(scale_y)
            }
        };

        // New content size after scaling
        let width = content_rect.width() * scale;
        let height = content_rect.height() * scale;

        // Center the content within the canvas
        let min = Point::new((size.width - width) / 2.0, (size.height - height) / 2.0);

        // Build the new canvas rect centered with content
        let rect = geo2d::Rect::new(min, min + geo2d::Point::new(width, height));

        Canvas {
            rect,
            content_rect,
            size,
        }
    }

    /// Return the ratio between canvas rect and content rect size.
    pub fn scale(&self) -> Scalar {
        (self.rect.width() / self.content_rect.width())
            .min(self.rect.height() / self.content_rect.height())
    }
}

impl From<geo2d::Rect> for Canvas {
    fn from(rect: geo2d::Rect) -> Self {
        Canvas::new(rect)
    }
}

/// Map something into a canvas coordinates.
pub trait MapToCanvas: Sized {
    /// Return mapped version.
    fn map_to_canvas(&mut self, canvas: &Canvas);
}

/// Scale scalar value.
impl MapToCanvas for Scalar {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        *self *= canvas.scale()
    }
}

impl MapToCanvas for (Scalar, Scalar) {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        let scale = canvas.scale();

        // Translate relative to content rect, scale, flip Y, and translate to destination canvas offset
        let x = (self.0 - canvas.content_rect.min().x) * scale + canvas.rect.min().x;
        let y = (canvas.content_rect.max().y - self.1) * scale + canvas.rect.min().y;

        *self = (x, y);
    }
}

impl MapToCanvas for Point {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        let mut xy = self.x_y();
        xy.map_to_canvas(canvas);
        *self = xy.into();
    }
}

impl MapToCanvas for Vec2 {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        let mut xy = (self.x, self.y);
        xy.map_to_canvas(canvas);
        *self = xy.into();
    }
}

impl MapToCanvas for Line {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.0.map_to_canvas(canvas);
        self.1.map_to_canvas(canvas);
    }
}

impl MapToCanvas for Rect {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        *self = Self::new(
            {
                let mut min = self.min().x_y();
                min.map_to_canvas(canvas);
                min
            },
            {
                let mut max = self.max().x_y();
                max.map_to_canvas(canvas);
                max
            },
        );
    }
}

impl MapToCanvas for Bounds2D {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        *self = self
            .rect()
            .map(|mut rect| {
                rect.map_to_canvas(canvas);
                rect.into()
            })
            .unwrap_or_default();
    }
}

impl MapToCanvas for Circle {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.radius.map_to_canvas(canvas);
        self.offset.map_to_canvas(canvas);
    }
}

impl MapToCanvas for Coord {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.x.map_to_canvas(canvas);
        self.y.map_to_canvas(canvas);
    }
}

impl MapToCanvas for LineString {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.0.iter_mut().for_each(|c| c.map_to_canvas(canvas))
    }
}

impl MapToCanvas for MultiLineString {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.iter_mut()
            .for_each(|line_string| line_string.map_to_canvas(canvas))
    }
}

impl MapToCanvas for Polygon {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.exterior_mut(|ext| ext.map_to_canvas(canvas));
        self.interiors_mut(|int| int.iter_mut().for_each(|int| int.map_to_canvas(canvas)));
    }
}

impl MapToCanvas for MultiPolygon {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.0
            .iter_mut()
            .for_each(|polygon| polygon.map_to_canvas(canvas));
    }
}

impl MapToCanvas for Geometries2D {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.iter_mut().for_each(|geo| geo.map_to_canvas(canvas));
    }
}

impl MapToCanvas for Geometry2D {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        match self {
            Geometry2D::LineString(line_string) => {
                line_string.map_to_canvas(canvas);
            }
            Geometry2D::MultiLineString(multi_line_string) => {
                multi_line_string.map_to_canvas(canvas);
            }
            Geometry2D::Polygon(polygon) => polygon.map_to_canvas(canvas),
            Geometry2D::MultiPolygon(multi_polygon) => {
                multi_polygon.map_to_canvas(canvas);
            }
            Geometry2D::Line(edge) => edge.map_to_canvas(canvas),
            Geometry2D::Collection(collection) => {
                collection.map_to_canvas(canvas);
            }
        }
    }
}

impl MapToCanvas for CenteredText {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.rect.map_to_canvas(canvas);
        self.font_size.map_to_canvas(canvas);
    }
}

impl MapToCanvas for Geometry {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        match self {
            Geometry::Geometry2D(geo2d) => geo2d.map_to_canvas(canvas),
            Geometry::Geometry3D(_) => todo!("Error handling"),
        }
    }
}

impl MapToCanvas for GeometryOutput {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.geometry.map_to_canvas(canvas)
    }
}

impl MapToCanvas for GeometryOutputs {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        *self = Self::from_iter(self.iter().map(|output| {
            let mut geo = output.geometry.clone();
            geo.map_to_canvas(canvas);
            std::sync::Arc::new(geo.into())
        }));
    }
}

impl MapToCanvas for GeometryNodeData {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        self.outputs.map_to_canvas(canvas);
    }
}

impl MapToCanvas for GeometryTree {
    fn map_to_canvas(&mut self, canvas: &Canvas) {
        fn recurse(mut node: GeometryNodeMut, canvas: &Canvas) {
            node.get_mut().map_to_canvas(canvas);
        }

        recurse(self.root_mut(), canvas)
    }
}
