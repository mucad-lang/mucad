// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Display and Debug traits for tree-like output.

use crate::{DisplayWithCtx, tree::NodeRef};

/// Formatting state passed down through tree nodes.
#[derive(Clone, Debug, Default)]
pub struct TreeState {
    /// String prefix applied to the current node's line.
    pub prefix: String,
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Trait for displaying a user-facing tree hierarchy.
pub trait TreeDisplay {
    fn tree_fmt(&self, f: &mut std::fmt::Formatter<'_>, state: TreeState) -> std::fmt::Result;
}

// =========================================================================
// TreeDisplay Implementation
// =========================================================================

impl<'a, Ctx, T> DisplayWithCtx<Ctx> for NodeRef<'a, T>
where
    T: DisplayWithCtx<Ctx>,
{
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        self.write_node_with_ctx(f, ctx, TreeState::default())
    }
}

impl<'a, T> TreeDisplay for NodeRef<'a, T>
where
    T: std::fmt::Display,
{
    fn tree_fmt(&self, f: &mut std::fmt::Formatter<'_>, state: TreeState) -> std::fmt::Result {
        self.write_node(f, state)
    }
}

impl<'a, T> std::fmt::Display for NodeRef<'a, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tree_fmt(f, TreeState::default())
    }
}
