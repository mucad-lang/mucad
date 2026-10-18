// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Identifier, SrcRef};
use mucad_lang_types::{
    Model, ModelTree, Value, arguments,
    model::{
        Arena, BooleanOp, Element, GetProperty, Properties, Property, PropertyType,
        element::BuiltinWorkpiece,
    },
};

#[test]
fn test_properties_basic_crud() {
    let mut props = Properties::new();

    assert!(props.is_empty());
    assert_eq!(props.len(), 0);
    assert!(!props.contains("radius"));

    // 1. Insert property
    let inserted = props.set_property(Property::input("radius", 10.5));

    assert_eq!(inserted.name, Identifier::from("radius"));
    assert_eq!(inserted.value, Value::from(10.5));
    assert_eq!(inserted.ty, PropertyType::Input);

    // 2. Query property
    assert_eq!(props.len(), 1);
    assert!(!props.is_empty());
    assert!(props.contains("radius"));

    let retrieved = props.get_property("radius").expect("Property should exist");
    assert_eq!(retrieved.value, Value::from(10.5));

    // 3. Update existing property
    props.set_property(Property::input("radius", 20.0));

    assert_eq!(props.len(), 1); // Length should remain 1
    let updated = props.get_property("radius").unwrap();
    assert_eq!(updated.value, Value::from(20.0));
    assert_eq!(updated.ty, PropertyType::Input);
}

#[test]
fn test_from_arguments() {
    let args = arguments!(width = 100.0, height = 50.0);

    // Convert Arguments -> Properties
    let props = Properties::from(args);

    assert_eq!(props.len(), 2);
    assert!(props.contains("width"));
    assert!(props.contains("height"));

    let width_prop = props.get_property("width").unwrap();
    assert_eq!(width_prop.value, Value::from(100.0));
    assert_eq!(width_prop.ty, PropertyType::Input);
    assert_eq!(width_prop.src_ref, SrcRef::default());
}

#[test]
fn test_deref_and_iterators() {
    let mut props = Properties::new();
    props.set_property(Property::input("x", 1.0));
    props.set_property(Property::input("y", 2.0));

    // Test Deref to BTreeMap methods (e.g. .keys(), .values())
    let keys: Vec<_> = props.keys().map(|id| id.to_string()).collect();
    assert_eq!(keys, vec!["x", "y"]);

    // Test IntoIterator for &Properties
    let mut count = 0;
    for (id, prop) in &props {
        assert_eq!(id, &prop.name);
        count += 1;
    }
    assert_eq!(count, 2);

    // Test IntoIterator for owned Properties
    let owned_items: Vec<(Identifier, Property)> = props.into_iter().collect();
    assert_eq!(owned_items.len(), 2);
}

#[test]
fn test_model_tree_get_property_recursive_and_filtering() {
    let mut arena = Arena::new();

    // Leaf Node 1: Input property ("radius")
    let mut leaf1_model = Model::default();
    leaf1_model
        .properties
        .set_property(Property::input("radius", 5.0));
    let leaf1 = arena.new_node(leaf1_model);

    // Leaf Node 2: Hidden property ("internal_id") & Output property ("area")
    let mut leaf2_model = Model::default();
    leaf2_model
        .properties
        .set_property(Property::hidden("internal_id", 42));
    leaf2_model
        .properties
        .set_property(Property::output("area", 78.5));
    let leaf2 = arena.new_node(leaf2_model);

    // Parent Node: Groups leaf1 and leaf2
    let group_node = arena.new_node(Model::from(Element::Group));
    group_node.append(leaf1, &mut arena);
    group_node.append(leaf2, &mut arena);

    // Root Node: BooleanOp wrapping the group
    let root = arena.new_node(Model::new(Element::BuiltinWorkpiece(
        BuiltinWorkpiece::BooleanOp(BooleanOp::Union),
    )));
    root.append(group_node, &mut arena);

    let tree = ModelTree { root, arena };

    // --- Assertions ---

    // 1. Should recursively find Input property in leaf1
    let radius_prop = tree.get_property_value("radius");
    assert_eq!(radius_prop, Value::from(5.0));

    let radius_prop = tree
        .get_properties_recursive("radius")
        .first()
        .cloned()
        .unwrap();
    assert_eq!(radius_prop.ty, PropertyType::Input);

    // 2. Should recursively find Output property in leaf2
    let area_prop = tree.get_property_value("area");
    assert_eq!(area_prop, Value::from(78.5));

    let area_prop = tree
        .get_properties_recursive("area")
        .first()
        .cloned()
        .unwrap();
    assert_eq!(area_prop.ty, PropertyType::Output);

    // 3. Should IGNORE Hidden properties in leaf2
    assert_eq!(
        tree.get_properties_recursive("internal_id").len(),
        0,
        "Hidden properties must be ignored during recursive lookup"
    );

    // 4. Non-existent property across whole tree
    assert_eq!(tree.get_properties_recursive("height").len(), 0);
}
