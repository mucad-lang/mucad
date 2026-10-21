// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model tree.

mod builder;
mod node;
mod ops;

use mucad_lang_base::{DisplayWithCtx, HashId, LookUpName, hash_id, tree};
pub use node::{ModelNodeId, Node, NodeExt, NodeMut, NodeRef};

pub use builder::{BuildModelTreeError, ModelTreeBuilder, ModelTreeBuilderMut};

use mucad_macros::Artifact;
use serde::{Deserialize, Serialize};

use crate::{
    List, Model, Ty, Type, Value,
    model::{self, Element, element::BuiltinWorkpiece},
};

/// A model tree with a root node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Artifact)]
pub struct ModelTree {
    pub root: ModelNodeId,
    pub arena: Arena,
}

impl ModelTree {
    pub fn new(root: impl Into<Model>) -> Self {
        let mut arena = Arena::default();
        let root = arena.new_node(root.into());
        Self { root, arena }
    }

    /// Compute the hash of each model from its element, properties, attributes
    /// and the hashes of its children, e.g. to cache its rendered geometry.
    ///
    /// Call it once the tree is final, e.g. after evaluation,
    /// because combining trees (e.g. replacing `@input`) changes the hashes.
    pub fn compute_hashes(&mut self) {
        fn compute(id: ModelNodeId, arena: &mut Arena) -> HashId {
            let children: Vec<_> = id.children(arena).collect();
            let child_hashes: Vec<_> = children
                .into_iter()
                .map(|child| compute(child, arena))
                .collect();

            let model = arena[id].get_mut();
            model.hash_id = hash_id!(model.element, model.properties, model.attr, child_hashes);
            model.hash_id
        }
        compute(self.root, &mut self.arena);
    }

    /// Create a replication node if there is more than one child in the list
    pub fn to_replication(mut children: Vec<ModelTree>) -> Self {
        match children.len() {
            1 => children.remove(0), // Return this tree if we have only one element.
            _ => {
                let mut tree = ModelTree::new(Element::Replication);
                tree.extend(children);
                tree
            }
        }
    }

    pub fn root<'a>(&'a self) -> NodeRef<'a> {
        NodeRef::new(self.root, &self.arena)
    }

    /// The model of the root node, e.g. to set its attributes.
    pub fn root_model_mut(&mut self) -> &mut Model {
        self.arena[self.root].get_mut()
    }

    /// Returns `true` if the model contains primitives (e.g. a circle), which can be rendered.
    ///
    /// E.g. a sketch which only checks values has no primitives.
    pub fn has_primitives(&self) -> bool {
        self.root().descendants().any(|node| {
            matches!(
                node.element,
                model::Element::BuiltinWorkpiece(BuiltinWorkpiece::Primitive(..))
            )
        })
    }
}

impl model::AttributeAccess for ModelTree {
    fn get_attribute(&self, name: impl AsRef<str>) -> Option<Value> {
        self.root().get_attribute(name)
    }
}

impl ModelTree {
    /// Combines `self` and `other` under a boolean operation (`Union`, `Intersection`, or `Difference`).
    ///
    /// If `self`'s root element is already a matching `BooleanOp`, it reuses `self`'s root
    /// and appends `other` to the inner `Group` child (flattening equal adjacent operations).
    /// Otherwise, it builds a new `ModelTree` with the structure:
    ///
    /// Root: ModelNode(BooleanOp)
    /// └── Group
    ///     ├── lhs (self)
    ///     └── rhs (other)
    pub fn apply_boolean_op(mut self, op: model::BooleanOp, rhs: Self) -> Self {
        // Check if `self`'s root is already a BooleanOp with the SAME operation
        if let model::Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(current_op)) =
            self.root().element
            && current_op == op
        {
            // Find the single child Group under the current root
            if let Some(group_ref) = self.root().into_group_child() {
                let group_id = group_ref.id;

                // Copy rhs's root and subtree into self's arena under the group
                let rhs_root_id = self.adopt_tree(rhs.root, &rhs.arena);
                group_id.append(rhs_root_id, &mut self.arena);

                return self;
            }
        }

        // --- Otherwise, construct the new tree structure ---
        let mut arena = Arena::new();

        // 1. Create the top-level BooleanOp root node
        let root_model = Model {
            element: model::Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(op)),
            ..Model::default()
        };
        let root = arena.new_node(root_model);

        // 2. Create the child Group node
        let group_model = Model {
            element: model::Element::Group,
            ..Model::default()
        };
        let group_id = arena.new_node(group_model);
        root.append(group_id, &mut arena);

        // 3. Adopt both self's root and rhs's root into the new arena under Group
        let lhs_root_id = tree::adopt_tree_to_arena(&mut arena, self.root, &self.arena);
        let rhs_root_id = tree::adopt_tree_to_arena(&mut arena, rhs.root, &rhs.arena);

        group_id.append(lhs_root_id, &mut arena);
        group_id.append(rhs_root_id, &mut arena);

        Self { root, arena }
    }

    /// Recursively copies a sub-tree from `source_arena` into `self.arena`.
    pub(crate) fn adopt_tree(
        &mut self,
        source_id: ModelNodeId,
        source_arena: &Arena,
    ) -> ModelNodeId {
        tree::adopt_tree_to_arena(&mut self.arena, source_id, source_arena)
    }

    pub fn append(&mut self, child: impl Into<ModelTree>) {
        let tree = child.into();
        let node_id = self.adopt_tree(tree.root, &tree.arena);
        self.root.append(node_id, &mut self.arena);
    }

    /// Returns a new `ModelTree` where every `Element::InputPlaceholder` node
    /// (and its descendants) is replaced with a deep copy of `input_model`.
    pub fn replace_input_placeholders(&self, input_model: &ModelTree) -> Self {
        let mut arena = Arena::new();

        // Recursively build the transformed tree starting from root
        let root = tree::replace_recursive(
            self.root,
            &self.arena,
            input_model.root,
            &input_model.arena,
            &mut arena,
            &|node| node.element == crate::model::element::Element::InputPlaceholder,
        );

        Self { root, arena }
    }

    /// If there are several properties
    ///
    pub fn get_property_value(&self, id: impl AsRef<str>) -> Value {
        let properties = self.get_properties_recursive(id);
        match properties.len() {
            0 => Value::None,
            1 => properties.first().unwrap().value.clone(),
            _ => List::from_iter(
                properties
                    .into_iter()
                    .map(|property| property.value.clone()),
            )
            .into(),
        }
    }

    /// Recursively searches all descendants in the model tree for `Input` or `Output`
    /// properties with the matching identifier.
    ///
    /// Ignores `Hidden` properties.
    pub fn get_properties_recursive(&self, id: impl AsRef<str>) -> Vec<&model::Property> {
        self._get_properties_recursive(self.root, id)
    }
}

impl std::hash::Hash for ModelTree {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root().hash(state);
    }
}

impl std::fmt::Display for ModelTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root().fmt(f)
    }
}

impl<Ctx: LookUpName> DisplayWithCtx<Ctx> for ModelTree {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        self.root().fmt_with_ctx(f, ctx)
    }
}

impl Ty for ModelTree {
    fn ty(&self) -> Type {
        Type::Model(self.root().output_type())
    }
}

/// `lhs | rhs` -> Union operation
impl std::ops::BitOr for ModelTree {
    type Output = ModelTree;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.apply_boolean_op(model::BooleanOp::Union, rhs)
    }
}

/// `lhs & rhs` -> Intersection operation
impl std::ops::BitAnd for ModelTree {
    type Output = ModelTree;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.apply_boolean_op(model::BooleanOp::Intersect, rhs)
    }
}

/// `lhs - rhs` -> Difference operation
impl std::ops::Sub for ModelTree {
    type Output = ModelTree;

    fn sub(self, rhs: Self) -> Self::Output {
        self.apply_boolean_op(model::BooleanOp::Difference, rhs)
    }
}

impl From<Value> for ModelTree {
    fn from(value: Value) -> Self {
        match value {
            Value::Model(model_tree) => {
                std::rc::Rc::try_unwrap(model_tree).unwrap_or_else(|rc| (*rc).clone())
            }
            value => ModelTree::new(crate::model::Element::Value(value)),
        }
    }
}

/// A copy of the subtree of a node, e.g. to export a part of a model.
impl From<NodeRef<'_>> for ModelTree {
    fn from(node: NodeRef<'_>) -> Self {
        let mut arena = Arena::default();
        let root = tree::adopt_tree_to_arena(&mut arena, node.id, node.arena);
        Self { root, arena }
    }
}

impl From<Model> for ModelTree {
    fn from(model: Model) -> Self {
        Self::new(model)
    }
}

impl Extend<ModelTree> for ModelTree {
    fn extend<T: IntoIterator<Item = ModelTree>>(&mut self, iter: T) {
        iter.into_iter().for_each(|child| self.append(child));
    }
}

pub type Arena = mucad_lang_base::tree::Arena<Model>;
