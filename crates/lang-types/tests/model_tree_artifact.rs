// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Artifact, ArtifactError, Identifier};
use mucad_lang_types::{
    ModelTree, arguments,
    model::{Element, Model, Properties},
};

/// Helper to construct a basic test `ModelTree`
fn create_test_model_tree() -> ModelTree {
    let mut tree = ModelTree::new(Model {
        name: Some(Identifier::from("RootModel")),
        properties: Properties::from(arguments!(width = 100.0, height = 50.0)),
        attr: Default::default(),
        element: Element::InputPlaceholder,
        creator: None,
        ..Default::default()
    });

    let child_node = tree.arena.new_node(Model {
        name: Some(Identifier::from("ChildModel")),
        properties: Properties::from(arguments!(radius = 12.5)),
        attr: Default::default(),
        element: Element::InputPlaceholder,
        creator: None,
        hash_id: Default::default(),
    });

    tree.root.append(child_node, &mut tree.arena);
    tree
}

#[test]
fn test_model_tree_ron_roundtrip() -> Result<(), ArtifactError> {
    let original_tree = create_test_model_tree();

    // 1. Serialize to RON artifact format
    let ron_str = original_tree.to_ron()?;

    println!("{ron_str}");

    // Verify the RON contains expected envelope header details
    assert!(ron_str.contains("ModelTree"));

    // 2. Deserialize from RON
    let deserialized_tree = ModelTree::from_ron(ron_str.as_str())?;

    // 3. Assert structural equality
    assert_eq!(original_tree, deserialized_tree);

    Ok(())
}

#[test]
fn test_model_tree_binary_roundtrip() -> Result<(), ArtifactError> {
    let original_tree = create_test_model_tree();

    // 1. Serialize to binary postcard artifact format
    let bytes = original_tree.to_binary()?;
    assert!(!bytes.is_empty());

    // 2. Deserialize from binary
    let deserialized_tree = ModelTree::from_binary(&bytes)?;

    // 3. Assert structural equality
    assert_eq!(original_tree, deserialized_tree);

    Ok(())
}
