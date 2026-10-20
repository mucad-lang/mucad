// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::BuiltinId;
use mucad_lang_types::{
    Arguments, Identifier, Length, Model, ModelTree, ModelType, Value, arguments,
    math::AffineTransform,
    model::{
        BooleanOp, Element, NodeExt,
        element::{self, BuiltinWorkpiece},
    },
    tuple,
};

mod model {
    use super::*;

    /// A Group model with name
    pub fn group(name: &str) -> Model {
        Model::new(Element::Group).with_name(name)
    }

    /// A Circle model with a radius
    pub fn circle(name: &str, radius: f64) -> Model {
        Model::new(element::BuiltinWorkpiece::Primitive(
            BuiltinId::from_name("__mu::geo2d::Circle"),
            ModelType::Geometry2D,
        ))
        .with_name(name)
        .with_properties(Arguments::from(tuple!(radius = Length::mm(radius))))
    }

    /// Helper to build a `__mu::ops::translate(x, y, z)` transform node
    pub fn translation(x_mm: f64, y_mm: f64, z_mm: f64) -> Model {
        let x = Length::mm(x_mm);
        let y = Length::mm(y_mm);
        let z = Length::mm(z_mm);

        let args = arguments!(x = x, y = y, z = z);
        Model::from(AffineTransform::Translation { x, y, z })
            .with_name("translate")
            .with_op_properties(args)
    }
}

#[test]
fn test_model_tree_creation_and_root() {
    let root_model = model::circle("root", 2.0);
    let tree = ModelTree::new(root_model);

    let root_ref = tree.root();

    assert_eq!(root_ref.name(), Some(&Identifier::from("root")));
}

#[test]
fn test_tree_hierarchy_and_children() {
    let mut tree = ModelTree::new(model::circle("parent", 1.0));

    tree.append(model::circle("child1", 2.0));
    tree.append(model::circle("child2", 3.0));

    // Verify traversal via root reference
    let root_ref = tree.root();
    let child_names: Vec<String> = root_ref
        .children()
        .filter_map(|node| node.name().map(|id| id.to_string()))
        .collect();

    assert_eq!(child_names, vec!["child1", "child2"]);
}

#[test]
fn test_deduce_output_type_fallback() {
    // Create a root model with an undetermined output type (e.g., a generic Group)
    let group_model = model::group("group");

    let mut tree = ModelTree::new(group_model);

    // Child model has a concrete output type
    let child_model = Model::new(BuiltinWorkpiece::Primitive(
        BuiltinId::from("__mu::geo2d::Circle"),
        ModelType::Geometry2D,
    ));
    tree.append(child_model);

    // The root should fall back to its child's type
    let root_ref = tree.root();
    let deduced_type = root_ref.deduce_output_type();

    assert_ne!(deduced_type, ModelType::NotDetermined);
}

#[test]
fn test_into_group_child() {
    let mut tree = ModelTree::new(model::group("parent_group"));

    tree.append(model::group("inner_group"));

    let root_ref = tree.root();
    let group_child = root_ref.into_group_child();

    println!("{root_ref}");

    // Should successfully unwrap the single nested group child
    assert!(group_child.is_some());
    assert_eq!(
        group_child.unwrap().name(),
        Some(&Identifier::from("inner_group"))
    );
}

/// Helper to extract child names under the inner group
fn get_group_child_names(tree: &ModelTree) -> Vec<String> {
    let root = tree.root();
    let group_node = root
        .into_group_child()
        .expect("Root should contain a single Group child");

    group_node
        .children()
        .filter_map(|child| child.name().map(|id| id.to_string()))
        .collect()
}

#[test]
fn test_basic_boolean_op_structure() {
    let a = ModelTree::new(model::circle("a", 2.0));
    let b = ModelTree::new(model::circle("b", 1.0));

    // a - b (Difference)
    let diff_tree = a - b;

    let root = diff_tree.root();

    // 1. Check root is BooleanOp::Difference
    assert_eq!(
        root.element,
        Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(BooleanOp::Difference))
    );

    // 2. Check group children are "a" and "b"
    let child_names = get_group_child_names(&diff_tree);
    assert_eq!(child_names, vec!["a", "b"]);
}

#[test]
fn test_flattening_same_boolean_op() {
    let a = ModelTree::new(model::circle("a", 1.0));
    let b = ModelTree::new(model::circle("b", 2.0));
    let c = ModelTree::new(model::circle("c", 3.0));

    // Chained Union: (a | b) | c
    let union_tree = a | b | c;

    // Verify root is single BooleanOp::Union
    assert_eq!(
        union_tree.root().element,
        Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(BooleanOp::Union))
    );

    // Verify all 3 leaves are flattened under the SAME inner group
    let child_names = get_group_child_names(&union_tree);
    assert_eq!(child_names, vec!["a", "b", "c"]);
}

#[test]
fn test_nested_different_boolean_ops() {
    let a = ModelTree::new(model::circle("a", 1.0));
    let b = ModelTree::new(model::circle("b", 2.0));
    let c = ModelTree::new(model::circle("c", 3.0));

    // Mixed operations: (a | b) & c -> Union inside Intersection
    let mixed_tree = (a | b) & c;

    // Root should be Intersection
    assert_eq!(
        mixed_tree.root().element,
        Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(BooleanOp::Intersect))
    );

    // Inner group of the root should have 2 children: [UnionNode, Leaf("c")]
    let root_group = mixed_tree
        .root()
        .into_group_child()
        .expect("Root must have group child");

    let children: Vec<_> = root_group.children().collect();
    assert_eq!(children.len(), 2);

    // First child is the nested Union operation
    assert_eq!(
        children[0].element,
        Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(BooleanOp::Union))
    );

    // Second child is leaf "c"
    assert_eq!(children[1].name(), Some(&Identifier::from("c")));
}

#[test]
fn test_subtree_adoption_preserves_hierarchy() {
    let mut parent_tree = ModelTree::new(model::circle("parent", 1.0));
    let child1 = ModelTree::new(model::circle("child1", 2.0));
    let child2 = ModelTree::new(model::circle("child2", 3.0));

    // Construct a deep subtree for LHS
    parent_tree.append(child1);
    parent_tree.append(child2);

    let rhs = ModelTree::new(model::circle("rhs", 4.0));

    // Combine deep parent_tree with simple rhs
    let combined = parent_tree | rhs;

    let root_group = combined.root().into_group_child().unwrap();
    let group_children: Vec<_> = root_group.children().collect();

    // First item should be "parent", which still contains its 2 sub-children
    let adopted_parent = group_children[0];
    assert_eq!(adopted_parent.name(), Some(&Identifier::from("parent")));

    let adopted_subchildren: Vec<_> = adopted_parent
        .children()
        .filter_map(|c| c.name().map(|id| id.to_string()))
        .collect();

    assert_eq!(adopted_subchildren, vec!["child1", "child2"]);
}

#[test]
fn test_replace_input_placeholders_list_expansion() {
    use mucad_lang_types::model::GetProperty;

    // 1. Build Replication template tree:
    // Replication
    //   ├── translate(0, 0, 0)
    //   │     └── InputPlaceholder
    //   ├── translate(0, 10, 0)
    //   │     └── InputPlaceholder
    //   ├── translate(10, 0, 0)
    //   │     └── InputPlaceholder
    //   └── translate(10, 10, 0)
    //         └── InputPlaceholder
    let mut template_tree =
        ModelTree::new(Model::new(Element::Replication).with_name("Replication"));

    let translations = [
        (0.0, 0.0, 0.0),
        (0.0, 10.0, 0.0),
        (10.0, 0.0, 0.0),
        (10.0, 10.0, 0.0),
    ];

    for (x, y, z) in translations {
        let trans_model = model::translation(x, y, z);
        let trans_id = template_tree.arena.new_node(trans_model);

        let placeholder_model = Model::new(Element::InputPlaceholder);
        let placeholder_id = template_tree.arena.new_node(placeholder_model);

        // Attach InputPlaceholder under translation, and translation under Replication
        trans_id.append(placeholder_id, &mut template_tree.arena);
        template_tree
            .root
            .append(trans_id, &mut template_tree.arena);
    }

    let circle_tree = ModelTree::new(model::circle("Circle", 5.0));

    // 3. Perform replacement
    let result_tree = template_tree.replace_input_placeholders(&circle_tree);

    // 4. Assertions
    let root = result_tree.root();
    assert_eq!(root.element, Element::Replication);

    let branches: Vec<_> = root.children().collect();
    assert_eq!(branches.len(), 4, "Should have 4 translation branches");

    for (idx, branch) in branches.iter().enumerate() {
        // Verify branch is still translate transform
        assert_eq!(branch.name(), Some(&Identifier::from("translate")));
        if let Element::BuiltinWorkpiece(BuiltinWorkpiece::AffineTransform(ref transform)) =
            branch.element
        {
            assert!(matches!(**transform, AffineTransform::Translation { .. }));
        } else {
            panic!(
                "Expected BuiltinWorkpiece::AffineTransform, got {:?}",
                branch.element
            );
        }

        // Verify children under translate
        let branch_children: Vec<_> = branch.children().collect();
        assert_eq!(
            branch_children.len(),
            1,
            "Branch {idx} should have exactly 1 child"
        );

        let replaced_child = branch_children[0];

        // Ensure InputPlaceholder is replaced by the Circle model
        assert_ne!(
            replaced_child.element,
            Element::InputPlaceholder,
            "Placeholder in branch {idx} was not replaced"
        );
        assert_eq!(replaced_child.name(), Some(&Identifier::from("Circle")));
        assert_eq!(
            replaced_child
                .properties
                .get_property("radius")
                .map(|p| &p.value),
            Some(&Value::from(Length::mm(5.0)))
        );
    }

    println!("{result_tree}")
}
