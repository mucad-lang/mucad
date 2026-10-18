// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Internal Model tree operations

use crate::model;

impl model::ModelTree {
    /// Looks up input and output properties in all descendants of the model tree.
    ///
    /// Recursive depth-first helper to search nodes in `self.arena`.
    pub(super) fn _get_properties_recursive(
        &self,
        current_id: model::ModelNodeId,
        target_id: impl AsRef<str>,
    ) -> Vec<&model::Property> {
        let node = model::NodeRef::new(current_id, &self.arena);

        let model = node.get();

        // 1. Check if the current node's model contains a matching Input/Output property
        if let Some(prop) = model.get_property(target_id.as_ref())
            && matches!(
                prop.ty,
                model::PropertyType::Input | model::PropertyType::Output
            )
        {
            return vec![prop];
        }

        // 2. Recursively search children
        node.children()
            .flat_map(|child| self._get_properties_recursive(child.id, target_id.as_ref()))
            .collect()
    }
}
