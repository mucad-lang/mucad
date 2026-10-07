// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Index-based tree node reference helpers.

pub use crate::tree::{Arena, NodeId};
use crate::{DisplayWithCtx, TreeState};

/// A convenience wrapper pairing a `NodeId` with a borrowed `Arena`.
pub struct NodeRef<'a, T> {
    pub id: NodeId,
    pub arena: &'a Arena<T>,
}

// Manual Copy: T does NOT need to implement Copy!
impl<'a, T> Copy for NodeRef<'a, T> {}

// Manual Clone (required whenever implementing Copy)
impl<'a, T> Clone for NodeRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> NodeRef<'a, T> {
    pub fn new(id: NodeId, arena: &'a Arena<T>) -> Self {
        Self { id, arena }
    }

    /// Access the underlying value payload.
    pub fn get(&self) -> &'a T {
        self.arena[self.id].get()
    }

    /// Get the parent node as a `NodeRef`.
    pub fn parent(&self) -> Option<NodeRef<'a, T>> {
        self.arena[self.id]
            .parent()
            .map(|parent_id| NodeRef::new(parent_id, self.arena))
    }
}

impl<'a, T> NodeRef<'a, T> {
    /// Helper to format a node and recurse through its children.
    pub fn write_node(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        tree_state: TreeState,
    ) -> std::fmt::Result
    where
        T: std::fmt::Display,
    {
        let content = self.get().to_string();
        let mut lines = content.lines();

        let prefix = tree_state.prefix;

        // 1. Output first line using the current prefix (may end with ├── or └── or be empty at root)
        if let Some(first_line) = lines.next() {
            writeln!(f, "{prefix}{first_line}")?;
        } else {
            writeln!(f, "{prefix}")?;
        }

        // 2. Compute continuation prefix for multiline content AND children.
        // Convert any trailing branch symbol into its continuation equivalent:
        // "├── " -> "│   "
        // "└── " -> "    "
        let continuation_prefix = if let Some(base) = prefix.strip_suffix("├── ") {
            format!("{base}│   ")
        } else if let Some(base) = prefix.strip_suffix("└── ") {
            format!("{base}    ")
        } else {
            prefix.clone()
        };

        let children: Vec<_> = self.children().collect();
        let count = children.len();

        // 3. Multiline lines use the continuation prefix
        for line in lines {
            if line.is_empty() && count > 0 {
                writeln!(f, "│")?;
                break;
            }

            let mut remainder = line;
            let mut indent_count = 0;

            // Strip every 4-space prefix and count how many times it appears
            while let Some(stripped) = remainder.strip_prefix("    ") {
                remainder = stripped;
                indent_count += 1;
            }

            if indent_count > 0 && count > 0 {
                let pipes = "│   ".repeat(indent_count);
                writeln!(f, "{continuation_prefix}{pipes}{remainder}")?;
            } else {
                writeln!(f, "{continuation_prefix}{line}")?;
            }
        }

        // 4. Recurse children using continuation_prefix + branch marker
        for (idx, child) in children.into_iter().enumerate() {
            let is_last = idx == count - 1;
            let branch = if is_last { "└── " } else { "├── " };

            let state = TreeState {
                prefix: format!("{continuation_prefix}{branch}"),
            };
            child.write_node(f, state)?;
        }

        Ok(())
    }

    /// Helper to format a node and recurse through its children.
    pub fn write_node_with_ctx<Ctx>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        ctx: &Ctx,
        tree_state: TreeState,
    ) -> std::fmt::Result
    where
        T: DisplayWithCtx<Ctx>,
    {
        let content = self.get().to_string_with_ctx(ctx);
        let mut lines = content.lines();

        let prefix = tree_state.prefix;

        // 1. Output first line using the current prefix (may end with ├── or └── or be empty at root)
        if let Some(first_line) = lines.next() {
            writeln!(f, "{prefix}{first_line}")?;
        } else {
            writeln!(f, "{prefix}")?;
        }

        // 2. Compute continuation prefix for multiline content AND children.
        // Convert any trailing branch symbol into its continuation equivalent:
        // "├── " -> "│   "
        // "└── " -> "    "
        let continuation_prefix = if let Some(base) = prefix.strip_suffix("├── ") {
            format!("{base}│   ")
        } else if let Some(base) = prefix.strip_suffix("└── ") {
            format!("{base}    ")
        } else {
            prefix.clone()
        };

        let children: Vec<_> = self.children().collect();
        let count = children.len();

        // 3. Multiline lines use the continuation prefix
        for line in lines {
            let mut remainder = line;
            let mut indent_count = 0;

            // Strip every 4-space prefix and count how many times it appears
            while let Some(stripped) = remainder.strip_prefix("    ") {
                remainder = stripped;
                indent_count += 1;
            }

            if indent_count > 0 && count > 0 {
                let pipes = "|   ".repeat(indent_count);
                writeln!(f, "{continuation_prefix}{pipes}{remainder}")?;
            } else {
                writeln!(f, "{continuation_prefix}{line}")?;
            }
        }

        // 4. Recurse children using continuation_prefix + branch marker
        for (idx, child) in children.into_iter().enumerate() {
            let is_last = idx == count - 1;
            let branch = if is_last { "└── " } else { "├── " };

            let state = TreeState {
                prefix: format!("{continuation_prefix}{branch}"),
            };
            child.write_node_with_ctx(f, ctx, state)?;
        }

        Ok(())
    }
}

impl<'a, T> std::ops::Deref for NodeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

/// Iterator methods.
impl<'a, T> NodeRef<'a, T> {
    /// Iterate over children wrapped as `NodeRef`s.
    pub fn children(&self) -> impl Iterator<Item = NodeRef<'a, T>> {
        let arena = self.arena;
        self.id
            .children(arena)
            .map(move |id| NodeRef::new(id, arena))
    }

    /// Iterate over ancestors wrapped as `NodeRef`s.
    pub fn ancestors(&self) -> impl Iterator<Item = NodeRef<'a, T>> {
        let arena = self.arena;
        self.id
            .ancestors(arena)
            .map(move |id| NodeRef::new(id, arena))
    }

    /// Iterate over ancestors wrapped as `NodeRef`s.
    pub fn descendants(&self) -> impl Iterator<Item = NodeRef<'a, T>> {
        let arena = self.arena;
        self.id
            .descendants(arena)
            .map(move |id| NodeRef::new(id, arena))
    }
}

pub struct NodeMut<'a, T> {
    pub id: NodeId,
    pub arena: &'a mut Arena<T>,
}

/// A mutable Node reference.
impl<'a, T> NodeMut<'a, T> {
    pub fn new(id: NodeId, arena: &'a mut Arena<T>) -> Self {
        Self { id, arena }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.arena[self.id].get_mut()
    }
}

/// Transformator methods.
impl<'a, T> NodeMut<'a, T> {
    /// Mutates the payload of all nodes in this subtree in-place.
    /// The closure receives a short-lived `NodeRef` (for inspecting topology)
    /// and a `&mut T` payload to update.
    pub fn transform<F>(&mut self, mut f: F)
    where
        F: FnMut(NodeRef<'_, T>, &mut T),
    {
        let node_ids: Vec<NodeId> = self.id.descendants(self.arena).collect();
        let arena_ptr = self.arena as *mut Arena<T>;
        node_ids.into_iter().for_each(|id| {
            // 1. Mutable reference to payload
            let payload = unsafe { (&mut *arena_ptr)[id].get_mut() };

            // 2. Both are passed together safely
            f(NodeRef::new(id, self.arena), payload);
        });
    }
}

impl<'a, T> std::ops::Deref for NodeMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.arena[self.id].get()
    }
}

impl<'a, T> std::ops::DerefMut for NodeMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indextree_macros::tree;

    #[derive(Debug, PartialEq, Eq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_transform_simple_payload_mutation() {
        // Construct tree using indextree_macros::tree!
        let mut arena = Arena::new();
        let root_id = tree!(&mut arena,
            TestData { name: "Root".into(), value: 10 } => {
                TestData { name: "Child".into(), value: 20 },
            }
        );

        let child_id = root_id.children(&arena).next().unwrap();
        let mut node_mut = NodeMut::new(root_id, &mut arena);

        // Mutate all payload values in the subtree
        node_mut.transform(|_node_ref, payload| {
            payload.value += 5;
        });

        assert_eq!(arena[root_id].get().value, 15);
        assert_eq!(arena[child_id].get().value, 25);
    }

    #[test]
    fn test_transform_using_topology_context() {
        // Construct hierarchy: root -> child -> grandchild
        let mut arena = Arena::new();
        let root_id = tree!(&mut arena,
            TestData { name: "Root".into(), value: 100 } => {
                TestData { name: "Child".into(), value: 50 } => {
                    TestData { name: "Grandchild".into(), value: 10 },
                }
            }
        );

        let child_id = root_id.children(&arena).next().unwrap();
        let grandchild_id = child_id.children(&arena).next().unwrap();

        let mut node_mut = NodeMut::new(root_id, &mut arena);

        // Use NodeRef topology inspection (counting ancestor depth)
        node_mut.transform(|node_ref, payload| {
            let depth = node_ref.ancestors().count() - 1;
            payload.value *= depth as i32;
        });

        // root has 0 ancestors -> 100 * 0 = 0
        assert_eq!(arena[root_id].get().value, 0);
        // child has 1 ancestor (root) -> 50 * 1 = 50
        assert_eq!(arena[child_id].get().value, 50);
        // grandchild has 2 ancestors (child, root) -> 10 * 2 = 20
        assert_eq!(arena[grandchild_id].get().value, 20);
    }
}
