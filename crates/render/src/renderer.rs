// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::SharedMut;
use microcad_lang_types::ModelTree;

use crate::{
    GeometryTree, RenderCache, RenderContext, RenderResolution, RenderResult, dispatch::RenderHooks,
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
    pub fn render(&mut self, model_tree: &ModelTree) -> RenderResult<GeometryTree> {
        let mut ctx = RenderContext::new(self, model_tree);
        ctx.execute()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
