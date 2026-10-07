// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model methods and trait implementations for rendering.

mod attribute;
mod cache;
mod context;
mod dispatch;
mod display;
mod output;
mod renderer;
mod resolution;
mod tree;

use cgmath::SquareMatrix;

pub use attribute::*;
pub use cache::*;
pub use context::*;
pub use tree::*;

use microcad_core::{self as core, traits::TransformAffine};
use microcad_lang_types::model::{ModelType, NodeExt};
pub use output::*;
pub use renderer::{ProgressTx, Renderer};
pub use resolution::RenderResolution;

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

use microcad_builtin::BuiltinError;

/// A trait that renders something with a reslution
pub trait RenderPrimitive<T = core::Geometry> {
    /// The render function.
    fn render_primitive(&self, resolution: &RenderResolution) -> T;

    /// A render hint to tell whether this geometry is independent from resolution.
    ///
    /// Example: a rectangle is resolution independent, but a circle is not.
    fn is_resolution_independent(&self) -> bool {
        false
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
