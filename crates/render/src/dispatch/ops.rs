use microcad_builtin::mu;
use microcad_core::{Extrude, Geometry, GeometryType, UnaryBooleanOp};

use crate::{GeometryOutput, GeometryOutputs, GeometryTree, Render, RenderContext, RenderResult};

impl Render for mu::ops::Difference {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| {
            let outputs = context.collect_outputs(tree).primary();
            let outputs = match outputs.ty() {
                GeometryType::Geometry2D => Geometry::geo2d(outputs.to_2d().difference()).into(),
                GeometryType::Geometry3D => Geometry::geo3d(outputs.to_3d().difference()).into(),
                GeometryType::Empty => GeometryOutputs::default(),
                GeometryType::Mixed => {
                    todo!("Error handling: Mixed geometry")
                }
            };

            Ok(outputs)
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
                1.0,
                1.0,
                cgmath::Rad(0.0),
            ))
            .into())
        })
    }
}
