// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_builtin::mu;
use microcad_core::{self as core, Extrude, Geometry, GeometryType, UnaryBooleanOp};
use microcad_lang_types::math::IntoFloat;

use crate::{GeometryOutput, GeometryOutputs, GeometryTree, Render, RenderContext, RenderResult};

fn boolean_op(
    tree: &GeometryTree,
    op: core::BooleanOp,
    context: &mut RenderContext,
) -> RenderResult {
    let outputs = context.collect_outputs(tree).primary();
    use GeometryType::*;
    let outputs = match outputs.ty() {
        Geometry2D => Geometry::geo2d(outputs.to_2d().boolean_op(op)).into(),
        Geometry3D => Geometry::geo3d(outputs.to_3d().boolean_op(op)).into(),
        Empty => GeometryOutputs::default(),
        Mixed => todo!("Error handling: Mixed geometry"),
    };

    Ok(outputs)
}

impl Render for mu::ops::Union {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| boolean_op(tree, core::BooleanOp::Union, context))
    }
}

impl Render for mu::ops::Subtract {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| boolean_op(tree, core::BooleanOp::Subtract, context))
    }
}

impl Render for mu::ops::Intersect {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| boolean_op(tree, core::BooleanOp::Intersect, context))
    }
}

impl Render for mu::ops::Hull {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| {
            use core::geo2d::ConvexHull;
            let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

            Ok(GeometryOutput::geo2d(multi_polygon.convex_hull()).into())
        })
    }
}

impl Render for mu::ops::Extrude {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| {
            let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

            Ok(GeometryOutput::from(multi_polygon.linear_extrude(
                microcad_core::Length::mm(self.height.as_mm()),
                self.scale_x.into_float(),
                self.scale_y.into_float(),
                self.twist.into_float(),
            ))
            .into())
        })
    }
}
