// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model tree iterators

use crate::{ModelNodeRef, model::Element};

/// Iterator over all descendants of multiplicities.
pub struct MultiplicityDescendants<'tree> {
    stack: Vec<ModelNodeRef<'tree>>,
}

impl<'tree> MultiplicityDescendants<'tree> {
    /// Create new descendants iterator
    pub fn new(model: ModelNodeRef<'tree>) -> Self {
        Self { stack: vec![model] }
    }
}

impl<'tree> Iterator for MultiplicityDescendants<'tree> {
    type Item = ModelNodeRef<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(model) = self.stack.pop() {
            if matches!(model.element, Element::Multiplicity) {
                // Expand but don't yield this node itself
                let children = model.children().collect::<Vec<_>>();
                self.stack.extend(children.iter().rev());
                continue;
            }
            // Return only non-multiplicity elements
            return Some(model);
        }
        None
    }
}
