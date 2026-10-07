// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use cgmath::SquareMatrix;
use microcad_core::{Bounds3D, Mat4};
use microcad_lang_types::{ModelNodeRef, ModelTree, math::ScalarF};

use crate::{GeometryNodeData, RenderResolution};

/// A model tree with a root node.
#[derive(Debug, Clone)]
pub struct GeometryTree {
    pub root: GeometryNodeId,
    pub arena: GeometryArena,
}

impl GeometryTree {
    pub fn new(model_tree: &ModelTree, render_resolution: RenderResolution) -> Self {
        let mut arena = GeometryArena::new();
        let mut tree = GeometryTree {
            root: Self::build_node(model_tree.root(), &mut arena),
            arena,
        };

        tree.prerender(model_tree, render_resolution);
        tree
    }

    /// Calculate world matrices and resolutions for each node.
    fn prerender(&mut self, model_tree: &ModelTree, render_resolution: RenderResolution) {
        let mut root_mut = self.root_mut();

        // Calculate world matrices
        root_mut.transform(|node, output| {
            let parent_matrix = node
                .parent()
                .map(|p| p.world_matrix)
                .unwrap_or(Mat4::identity());
            output.world_matrix = parent_matrix * output.local_matrix;
        });

        root_mut.resolution = Some(render_resolution);
        root_mut.transform(|node, output| {
            if let Some(parent) = node.parent() {
                use microcad_lang_types::model::attribute::ResolutionAttribute;
                let parent_resolution = parent.resolution.clone().unwrap_or_default();
                let resolution = match node.model(model_tree).resolution() {
                    Some(resolution_attribute) => RenderResolution {
                        linear: match resolution_attribute {
                            ResolutionAttribute::Absolute(linear) => linear.0.to_num(),
                            ResolutionAttribute::Relative(factor) =>
                            // Example: A relative resolution of 200% scales an absolution resolution from 0.1mm to 0.5mm.
                            {
                                parent_resolution.linear / factor.to_num::<ScalarF>()
                            }
                        },
                    },
                    None => parent_resolution,
                };

                output.resolution = Some(resolution);
            }
        });
    }

    /// Helper method to recursively transform ModelNodeRef into GeometryNodeId
    fn build_node(model_node: ModelNodeRef<'_>, arena: &mut GeometryArena) -> GeometryNodeId {
        // 1. Transform the ModelOutput / Model data into a RenderOutput
        let render_output = GeometryNodeData::new(model_node);

        // 2. Insert the new node into the RenderArena
        let current_node_id = arena.new_node(render_output);

        // 3. Traverse and append all children recursively
        for child_ref in model_node.children() {
            let child_node_id = Self::build_node(child_ref, arena);
            current_node_id.append(child_node_id, arena);
        }

        current_node_id
    }

    pub fn root<'a>(&'a self) -> GeometryNodeRef<'a> {
        GeometryNodeRef::new(self.root, &self.arena)
    }

    pub fn root_mut<'a>(&'a mut self) -> GeometryNodeMut<'a> {
        GeometryNodeMut::new(self.root, &mut self.arena)
    }

    pub fn bounds(&self) -> Bounds3D {
        self.root().outputs.bounds().clone()
    }
}

pub type GeometryArena = microcad_lang_base::tree::Arena<GeometryNodeData>;
pub type GeometryNode = microcad_lang_base::tree::Node<GeometryNodeData>;
pub type GeometryNodeRef<'a> = microcad_lang_base::tree::NodeRef<'a, GeometryNodeData>;
pub type GeometryNodeMut<'a> = microcad_lang_base::tree::NodeMut<'a, GeometryNodeData>;
pub type GeometryNodeId = microcad_lang_base::tree::NodeId;
