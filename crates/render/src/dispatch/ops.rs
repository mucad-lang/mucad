// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_builtin::mu;
use microcad_core::{self as core, CalcBounds2D, Extrude, Geometry, GeometryType, UnaryBooleanOp};
use microcad_lang_types::{math::IntoFloat, model::ModelType};

use crate::{
    GeometryOutput, GeometryOutputs, GeometryTree, Render, RenderContext, RenderError, RenderResult,
};

fn boolean_op(
    tree: &GeometryTree,
    op: core::BooleanOp,
    context: &mut RenderContext,
) -> RenderResult {
    let outputs = context.collect_outputs(tree).primary();
    use GeometryType::*;

    Ok(match outputs.ty() {
        Geometry2D => Geometry::geo2d(outputs.to_2d().boolean_op(op)).into(),
        Geometry3D => Geometry::geo3d(outputs.to_3d().boolean_op(op)).into(),
        Empty => GeometryOutputs::default(),
        Mixed => todo!("Error handling: Mixed geometry"),
    })
}

impl Render for mu::ops::Union {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        boolean_op(tree, core::BooleanOp::Union, context)
    }
}

impl Render for mu::ops::Subtract {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        boolean_op(tree, core::BooleanOp::Subtract, context)
    }
}

impl Render for mu::ops::Intersect {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        boolean_op(tree, core::BooleanOp::Intersect, context)
    }
}

impl Render for mu::ops::Hull {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::geo2d::ConvexHull;
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

        Ok(GeometryOutput::geo2d(multi_polygon.convex_hull()).into())
    }
}

impl Render for mu::ops::Extrude {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

        Ok(GeometryOutput::from(multi_polygon.linear_extrude(
            microcad_core::Length::mm(self.height.as_mm()),
            self.scale_x.into_float(),
            self.scale_y.into_float(),
            self.twist.into_float(),
        ))
        .into())
    }
}

impl Render for mu::ops::Revolve {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();
        let radius = multi_polygon.calc_bounds_2d().radius();

        Ok(GeometryOutput::from(multi_polygon.revolve_extrude(
            self.angle.into_float(),
            context.current_resolution(tree).circular_segments(radius) as usize,
        ))
        .into())
    }
}

impl Render for mu::ops::Buffer {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::geo2d::Buffer;
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

        Ok(GeometryOutput::geo2d(multi_polygon.buffer(self.distance.as_mm())).into())
    }
}

impl Render for mu::ops::Center {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::traits::Center;
        let outputs = context.collect_outputs(tree).primary();
        use GeometryType::*;

        Ok(match outputs.ty() {
            Geometry2D => Geometry::geo2d(outputs.to_2d()).center().into(),
            Geometry3D => Geometry::geo3d(outputs.to_3d()).center().into(),
            Empty => GeometryOutputs::default(),
            Mixed => todo!("Error handling: Mixed geometry"),
        })
    }
}

/// Apply `geo2d` or `geo3d` to the geometries of the children, depending on their type.
fn map_geometries(
    tree: &GeometryTree,
    context: &mut RenderContext,
    geo2d: impl FnOnce(core::Geometries2D) -> core::Geometries2D,
    geo3d: impl FnOnce(core::Geometries3D) -> core::Geometries3D,
) -> RenderResult<GeometryOutputs> {
    let outputs = context.collect_outputs(tree).primary();
    use GeometryType::*;

    match outputs.ty() {
        Geometry2D => Ok(Geometry::geo2d(geo2d(outputs.to_2d())).into()),
        Geometry3D => Ok(Geometry::geo3d(geo3d(outputs.to_3d())).into()),
        Empty => Ok(GeometryOutputs::default()),
        Mixed => Err(RenderError::InvalidOutputType(ModelType::Any)),
    }
}

impl Render for mu::ops::Align {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::{Align2D, Align3D};
        let (x, y, z) = (
            self.x.into_float(),
            self.y.into_float(),
            self.z.into_float(),
        );
        let spacing = core::Length::mm(self.spacing.as_mm());

        map_geometries(
            tree,
            context,
            |geometries| geometries.align_2d(core::Vec2::new(x, y), spacing),
            |geometries| geometries.align_3d(core::Vec3::new(x, y, z), spacing),
        )
    }
}

impl Render for mu::ops::DistributeGrid {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::traits::DistributeGrid;
        let (x, y) = (self.x.as_mm(), self.y.as_mm());
        let rect = core::Rect::new((x, y), (x + self.width.as_mm(), y + self.height.as_mm()));
        let (rows, columns) = (self.rows.to_num(), self.columns.to_num());

        map_geometries(
            tree,
            context,
            |geometries| geometries.distribute_grid(rect, rows, columns),
            |geometries| geometries.distribute_grid(rect, rows, columns),
        )
    }
}

/// The line (2D) or plane (3D) of a mirror or reflection through a point with a normal.
fn mirror_plane(
    (x, y, z): (core::Scalar, core::Scalar, core::Scalar),
    (nx, ny, nz): (core::Scalar, core::Scalar, core::Scalar),
) -> (core::geo2d::Line, core::geo3d::Plane) {
    use core::geo2d::Point;
    (
        core::geo2d::Line(Point::new(x, y), Point::new(x - ny, y + nx)),
        core::geo3d::Plane {
            p: core::Vec3::new(x, y, z),
            n: core::Vec3::new(nx, ny, nz),
        },
    )
}

impl Render for mu::ops::Mirror {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::{Mirror2D, Mirror3D};
        let (line, plane) = mirror_plane(
            (self.x.as_mm(), self.y.as_mm(), self.z.as_mm()),
            (
                self.nx.into_float(),
                self.ny.into_float(),
                self.nz.into_float(),
            ),
        );

        map_geometries(
            tree,
            context,
            |geometries| geometries.mirror_2d(&line),
            |geometries| geometries.mirror_3d(&plane),
        )
    }
}

impl Render for mu::ops::Reflect {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::{Reflect2D, Reflect3D};
        let (line, plane) = mirror_plane(
            (self.x.as_mm(), self.y.as_mm(), self.z.as_mm()),
            (
                self.nx.into_float(),
                self.ny.into_float(),
                self.nz.into_float(),
            ),
        );

        map_geometries(
            tree,
            context,
            |geometries| geometries.reflect_2d(&line),
            |geometries| geometries.reflect_3d(&plane),
        )
    }
}

impl Render for mu::ops::Spiralize {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();
        let mut bounds = multi_polygon.calc_bounds_2d();
        bounds.extend_by_point(core::Vec2::new(0.0, 0.0));
        let angle: core::Angle = self.angle.into_float();

        Ok(GeometryOutput::from(
            multi_polygon.spiralize(
                self.height.as_mm(),
                self.inner_radius.as_mm(),
                self.outer_radius.as_mm(),
                angle.0 / std::f64::consts::TAU,
                context
                    .current_resolution(tree)
                    .circular_segments(bounds.max_extent()) as usize,
            ),
        )
        .into())
    }
}

impl Render for mu::ops::Multiply {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let outputs = context.collect_outputs(tree).primary();
        let n: usize = self.n.to_num::<i64>().max(0) as usize;

        Ok(GeometryOutputs::from_iter(
            (0..n).flat_map(|_| outputs.iter().cloned()),
        ))
    }
}
