// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model methods and trait implementations for rendering.

mod attribute;
mod cache;
mod context;
mod diag;
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
pub use diag::{RenderInfo, RenderIssue, RenderWarning};
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

    /// A built-in has no render hook, its children are rendered instead.
    #[error("Built-in `{builtin}` cannot be rendered")]
    #[diagnostic(help("Its children are rendered instead."))]
    MissingRenderHook {
        builtin: String,
        #[label("No render hook")]
        src_ref: SrcRef,
    },
}

impl SrcReferrer for RenderError {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::MissingRenderHook { src_ref, .. } => *src_ref,
            Self::BuiltinError(err) => err.src_ref(),
            Self::InvalidOutputType(_) | Self::NothingToRender => SrcRef::none(),
        }
    }
}

use microcad_builtin::BuiltinError;
use microcad_lang_base::{LookUpName, SrcRef, SrcReferrer};

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
        Ok(self
            .render_primitive(&context.current_resolution(tree))
            .into())
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
        // Reported before the cache look up, to report it on each render.
        let model = context.model(tree);
        let hook = model
            .builtin_id()
            .map(|builtin_id| (builtin_id, context.renderer.hooks.get(builtin_id)));
        if let Some((builtin_id, None)) = hook {
            context.push_error(RenderError::MissingRenderHook {
                builtin: microcad_builtin::BuiltinLibrary::new()
                    .look_up_built_in_name(&builtin_id)
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| builtin_id.to_string()),
                src_ref: model
                    .creator
                    .as_ref()
                    .map(|creator| creator.src_ref)
                    .unwrap_or_else(SrcRef::none),
            });
        }

        context.update(self.cache_key(), |_node, context| {
            let mut outputs = match hook {
                Some((_, Some(hook))) => hook(tree, context)?,
                Some((_, None)) | None => context.collect_outputs(tree).primary(),
            };

            if !self.local_matrix.is_identity() {
                outputs.transform_affine(&self.local_matrix);
            }
            Ok(outputs)
        })
    }
}
