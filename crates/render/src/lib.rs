// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model methods and trait implementations for rendering.

mod attribute;
mod cache;
mod context;
mod display;
mod output;
mod render;
mod tree;

use cgmath::SquareMatrix;
use microcad_hash::HashMap;

pub use attribute::*;
pub use cache::*;
pub use context::*;
pub use tree::*;

use microcad_core::{
    self as core, Extrude, Geometry, Geometry2D, Scalar, UnaryBooleanOp, traits::TransformAffine,
};
use microcad_lang_types::model::{ModelType, NodeExt};
pub use output::*;
pub use render::{RenderPrimitive, RenderResolution};

use miette::Diagnostic;
use thiserror::Error;

/// An error that occurred during rendering.
#[derive(Debug, Error, Diagnostic)]
pub enum RenderError {
    /// Invalid output type.
    #[error("Invalid output type: {0}")]
    InvalidOutputType(ModelType),

    #[error("Builtin error")]
    BuiltinError(#[from] BuiltinError),

    /// Nothing to render.
    #[error("Nothing to render")]
    NothingToRender,
}

use microcad_builtin::{BuiltinConstruct, BuiltinError, BuiltinId, mu};

/// Built-in execution function signature
pub type RenderFn = fn(&GeometryTree, &mut RenderContext) -> Result<GeometryOutputs, RenderError>;

#[derive(Debug, Default)]
pub struct RenderHooks {
    hooks: HashMap<BuiltinId, RenderFn>,
}

/// Builder methods.
impl RenderHooks {
    pub fn new() -> Self {
        let mut hooks = RenderHooks::default();
        hooks.insert::<mu::geo2d::Circle>();
        hooks.insert::<mu::geo2d::Rect>();
        hooks.insert::<mu::ops::Difference>();
        hooks.insert::<mu::ops::Extrude>();
        hooks
    }
}

impl RenderHooks {
    pub fn insert<C: BuiltinConstruct + Render>(&mut self) {
        self.hooks.insert(C::ITEM.id(), |tree, ctx| {
            C::from_model(ctx.model(tree).get())?.render(tree, ctx)
        });
    }

    pub fn get(&self, id: BuiltinId) -> Option<RenderFn> {
        self.hooks.get(&id).cloned()
    }
}

impl RenderPrimitive for mu::geo2d::Circle {
    fn render_primitive(&self, resolution: &RenderResolution) -> Geometry {
        let radius: Scalar = self.radius.as_mm();
        let n = resolution.circular_segments(radius);
        Geometry2D::Polygon(microcad_core::Circle::circle_polygon(radius, n)).into()
    }
}

impl RenderPrimitive for mu::geo2d::Rect {
    fn render_primitive(&self, _: &RenderResolution) -> Geometry {
        let (x, y, w, h) = (
            self.x.as_mm(),
            self.y.as_mm(),
            self.width.as_mm(),
            self.height.as_mm(),
        );
        Geometry2D::Polygon(core::geo2d::Rect::new((x, y), (x + w, y + h)).to_polygon()).into()
    }
}

impl<T: RenderPrimitive> Render for T {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_, context| {
            Ok(self
                .render_primitive(&context.current_resolution(tree))
                .into())
        })
    }
}

impl Render for mu::ops::Difference {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| {
            let outputs = context.collect_outputs(tree).primary();
            let outputs = match outputs.ty() {
                microcad_core::GeometryType::Geometry2D => {
                    GeometryOutputs::from(Geometry::geo2d(outputs.to_2d().difference()))
                }
                microcad_core::GeometryType::Geometry3D => {
                    GeometryOutputs::from(Geometry::geo3d(outputs.to_3d().difference()))
                }
                microcad_core::GeometryType::Empty => GeometryOutputs::default(),
                microcad_core::GeometryType::Mixed => {
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

/// A result from rendering a model.
pub type RenderResult<T = GeometryOutputs> = Result<T, RenderError>;

/// The render trait.
pub trait Render<T = GeometryOutputs> {
    /// Render method.
    fn render(&self, tree: &GeometryTree, context: &mut RenderContext) -> RenderResult<T>;
}

impl Render for GeometryNodeData {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        context.update(|_node, context| {
            let model = context.model(tree);

            let mut outputs = match model
                .builtin_id()
                .and_then(|builtin_id| context.renderer.hooks.get(builtin_id))
            {
                Some(hook) => hook(tree, context)?,
                None => context.collect_outputs(tree).primary(),
            };

            if !self.local_matrix.is_identity() {
                outputs.transform_affine(&self.local_matrix);
            }
            Ok(outputs)
        })
    }
}
