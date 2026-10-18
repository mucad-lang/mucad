// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{
    DiagRenderOptions, Diagnostics, IssueResult, PushIssue, RenderDiag, SharedMut,
};
use mucad_lang_types::ModelTree;

use crate::{
    GeometryTree, RenderCache, RenderContext, RenderIssue, RenderResolution, dispatch::RenderHooks,
};

/// Our progress sender.
pub type ProgressTx = std::sync::mpsc::Sender<f32>;

/// The rendering engine setup and shared resources.
pub struct Renderer {
    pub resolution: RenderResolution,
    pub cache: Option<SharedMut<RenderCache>>,
    pub hooks: RenderHooks,
    pub progress_tx: Option<ProgressTx>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            resolution: RenderResolution::default(),
            cache: None,
            hooks: RenderHooks::new(),
            progress_tx: None,
        }
    }

    /// Override the default render resolution.
    pub fn with_resolution(mut self, resolution: RenderResolution) -> Self {
        self.resolution = resolution;
        self
    }

    /// Render cache.
    pub fn with_cache(mut self, cache: SharedMut<RenderCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// With progress tracker.
    pub fn with_progress(mut self, progress: ProgressTx) -> Self {
        self.progress_tx = Some(progress);
        self
    }
}

impl Renderer {
    /// Execute rendering for a given model tree.
    ///
    /// Errors which do not stop rendering (e.g. a missing render hook) are returned with the
    /// geometry tree.
    pub fn render(&mut self, model_tree: &ModelTree) -> IssueResult<GeometryTree, RenderIssue> {
        let mut ctx = RenderContext::new(self, model_tree);
        match ctx.execute() {
            Ok(tree) => Ok((tree, ctx.issues)),
            Err(err) => {
                let mut issues = ctx.issues;
                issues.push_err(err);
                Err(issues)
            }
        }
    }
}

/// The renderer has no sources, so diagnostics are rendered without source code,
/// e.g. render errors refer to the source code of the compiled models.
impl RenderDiag for Renderer {
    fn render_diag_with(
        &self,
        f: &mut dyn std::fmt::Write,
        diagnostics: &Diagnostics,
        options: &DiagRenderOptions,
    ) -> std::fmt::Result {
        diagnostics
            .iter()
            .try_for_each(|diag| diag.render_report(f, options))
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
