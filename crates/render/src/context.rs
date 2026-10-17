// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render context

use microcad_hash::HashId;
use microcad_lang_types::{ModelNodeRef, ModelTree};

use crate::{
    GeometryNodeId, GeometryNodeRef, GeometryOutputs, GeometryTree, Render, RenderResolution,
    RenderResult, Renderer,
};

/// The render context.
///
/// Keeps a stack of model nodes and the render cache.
pub struct RenderContext<'a, 'tree> {
    pub(crate) renderer: &'a mut Renderer,

    pub(crate) model_tree: &'tree ModelTree,

    /// Model stack.
    stack: Vec<GeometryNodeId>,

    /// The number of models to be rendered.
    models_to_render: usize,

    /// The number of model that been been rendered.
    models_rendered: usize,
}

/// Builder methods
impl<'a, 'tree> RenderContext<'a, 'tree> {
    /// Initialize context with a model tree.
    pub(crate) fn new(renderer: &'a mut Renderer, model_tree: &'tree ModelTree) -> Self {
        Self {
            renderer,
            model_tree,
            stack: vec![],
            models_rendered: 0,
            models_to_render: model_tree.root().descendants().count(),
        }
    }

    pub(crate) fn execute(&mut self) -> RenderResult<GeometryTree> {
        let mut tree = GeometryTree::new(self.model_tree, self.renderer.resolution.clone());
        /// Recursive DFS helper: processes children first, then renders the current node.
        fn recurse(
            node_id: GeometryNodeId,
            tree: &mut GeometryTree,
            context: &mut RenderContext,
        ) -> RenderResult<()> {
            // 1. Recurse down into all children first (Leaves are reached first)
            let children: Vec<_> = node_id.children(&tree.arena).collect();
            for child_id in children {
                recurse(child_id, tree, context)?;
            }

            context.stack.push(node_id);
            // 2. Render current node after all children have completed
            let render_output = tree.arena[node_id].get();
            let outputs = render_output.render(tree, context)?;
            context.step();

            let render_output = tree.arena[node_id].get_mut();
            render_output.outputs = outputs;

            context.stack.pop();

            Ok(())
        }
        recurse(tree.root, &mut tree, self)?;

        Ok(tree)
    }
}

impl<'a, 'tree> RenderContext<'a, 'tree> {
    /// Make a single progress step. A progress signal is sent with each new percentage.
    pub(crate) fn step(&mut self) {
        let old_percent = self.progress_in_percent();
        self.models_rendered += 1;
        let new_percent = self.progress_in_percent();

        // Check if integer percentage increased
        if (old_percent.floor() as u32) < (new_percent.floor() as u32)
            && let Some(progress_tx) = &mut self.renderer.progress_tx
        {
            progress_tx.send(new_percent).expect("No error");
        }
    }

    /// Return render progress in percent.
    pub fn progress_in_percent(&self) -> f32 {
        (self.models_rendered as f32 / self.models_to_render as f32) * 100.0
    }

    /// Collect outputs from children of current node
    pub fn collect_outputs(&self, tree: &GeometryTree) -> GeometryOutputs {
        GeometryOutputs::from_iter(
            self.current_node(tree)
                .children()
                .flat_map(|child| child.get().outputs.iter().cloned()),
        )
    }

    /// Return the outputs with the cache key `hash` from the render cache,
    /// or compute them with `f` and insert them into the cache.
    pub fn update(
        &mut self,
        hash: HashId,
        f: impl FnOnce(GeometryNodeId, &mut RenderContext) -> RenderResult<GeometryOutputs>,
    ) -> RenderResult<GeometryOutputs> {
        let geo = self.geo_node();
        let Some(cache) = self.renderer.cache.clone() else {
            return f(geo, self);
        };

        // The lock is not held while computing, which may take long.
        if let Some(outputs) = cache.with_write(|cache| cache.get(&hash).cloned()) {
            return Ok(outputs);
        }
        let (outputs, cost) = self.call_with_cost(geo, f)?;
        cache.with_write(|cache| {
            cache.insert_with_cost(hash, outputs.clone(), cost);
        });
        Ok(outputs)
    }

    /// Return current render resolution.
    pub fn current_resolution(&self, tree: &GeometryTree) -> RenderResolution {
        self.current_node(tree)
            .resolution
            .as_ref()
            .cloned()
            .unwrap_or_default()
    }

    // Return the generated item and the number of milliseconds.
    fn call_with_cost(
        &mut self,
        geo_node: GeometryNodeId,
        f: impl FnOnce(GeometryNodeId, &mut RenderContext) -> RenderResult<GeometryOutputs>,
    ) -> RenderResult<(GeometryOutputs, f64)> {
        use std::time::Instant;
        let start = Instant::now();

        let r = f(geo_node, self)?;

        let duration = start.elapsed();
        Ok((r, (duration.as_nanos() as f64) / 1_000_000.0))
    }

    pub fn geo_node(&self) -> GeometryNodeId {
        self.stack.last().copied().unwrap()
    }

    pub fn current_node<'b>(&self, tree: &'b GeometryTree) -> GeometryNodeRef<'b> {
        GeometryNodeRef::new(self.geo_node(), &tree.arena)
    }

    pub fn model(&self, tree: &GeometryTree) -> ModelNodeRef<'tree> {
        let node = self.current_node(tree);
        node.get().model(self.model_tree)
    }
}
