// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model node.

use mucad_lang_base::{BuiltinId, Identifier};

use crate::{Model, ModelType, model};

pub type Node = mucad_lang_base::tree::Node<Model>;
pub type NodeRef<'a> = mucad_lang_base::tree::NodeRef<'a, Model>;
pub type NodeMut<'a> = mucad_lang_base::tree::NodeMut<'a, Model>;
pub type ModelNodeId = mucad_lang_base::tree::NodeId;

/// Extension trait for [`SymbolNode`] .
pub trait NodeExt<'a> {
    fn name(&self) -> Option<&Identifier>;

    fn deduce_output_type(&self) -> ModelType;

    fn into_group_child(self) -> Option<NodeRef<'a>>;

    fn builtin_id(&self) -> Option<BuiltinId>;
}

impl<'a> NodeExt<'a> for NodeRef<'a> {
    fn name(&self) -> Option<&Identifier> {
        self.name.as_ref()
    }

    /// Deduce output type from element or children.
    fn deduce_output_type(&self) -> ModelType {
        let output_type = self.element.output_type();

        if output_type == ModelType::NotDetermined {
            // Fallback: iterate over children and deduce
            for child in self.children() {
                let child_type = child.deduce_output_type();
                if child_type != ModelType::NotDetermined {
                    return child_type;
                }
            }
        }

        output_type
    }

    /// Return inner group child if this model only contains a single group child.
    ///
    /// Useful for operations like `subtract() {}` or `hull() {}` to unwrap nested groups.
    fn into_group_child(self) -> Option<NodeRef<'a>> {
        let mut children = self.children();
        let first_child = children.next()?;

        // Ensure it's the ONLY child
        if children.next().is_none() && matches!(first_child.element, model::Element::Group) {
            Some(first_child)
        } else {
            None
        }
    }

    fn builtin_id(&self) -> Option<BuiltinId> {
        match &self.element {
            model::Element::BuiltinWorkpiece(builtin_workpiece) => builtin_workpiece.builtin_id(),
            _ => None,
        }
    }
}
