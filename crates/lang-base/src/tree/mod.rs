// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Index-based tree, based in crate `indextree`.

mod node_ref;
mod tree_display;

pub use indextree::*;

pub use node_ref::{NodeMut, NodeRef};

pub use tree_display::{TreeDisplay, TreeState};

/// Static helper to copy a sub-tree from `source_arena` into a target `ModelArena`.
pub fn adopt_tree_to_arena<Target, Source>(
    target_arena: &mut Arena<Target>,
    source_id: NodeId,
    source_arena: &Arena<Source>,
) -> NodeId
where
    Source: Clone,
    Target: From<Source>,
{
    let source_node = NodeRef::new(source_id, source_arena);

    // Create matching node in target arena
    let new_id = target_arena.new_node(source_node.get().clone().into());

    // Traverse children recursively
    for child in source_node.children() {
        let child_id = adopt_tree_to_arena(target_arena, child.id, source_arena);
        new_id.append(child_id, target_arena);
    }

    new_id
}

/// Recursive helper to reconstruct the tree, replacing placeholders.
pub fn replace_recursive<Target, Source, F>(
    current_id: NodeId,
    source_arena: &Arena<Source>,
    input_model_root: NodeId,
    input_model_arena: &Arena<Source>,
    target_arena: &mut Arena<Target>,
    pred: &F,
) -> NodeId
where
    Source: Clone,
    Target: From<Source>,
    F: Fn(&NodeRef<Source>) -> bool,
{
    let current_node = NodeRef::new(current_id, source_arena);
    if pred(&current_node) {
        // Replace this node AND its descendants with a full copy of `input_model`
        return adopt_tree_to_arena(target_arena, input_model_root, input_model_arena);
    }

    // 1. Copy the current node content into the target arena
    let new_id = target_arena.new_node(current_node.get().clone().into());

    // 2. Process children recursively and attach them
    for child in current_node.children() {
        let new_child_id = replace_recursive(
            child.id,
            source_arena,
            input_model_root,
            input_model_arena,
            target_arena,
            pred,
        );
        new_id.append(new_child_id, target_arena);
    }

    new_id
}
