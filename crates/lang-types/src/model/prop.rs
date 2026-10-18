// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model properties type.

use mucad_lang_base::{Identifier, SrcRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Deref;

use crate::{Arguments, Ty, Value};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum PropertyType {
    /// An input property, created from workbenches base parameters.
    Input,
    /// An output property, created by each assignment with the `prop` keyword.
    Output,
    /// A hidden property that can be displayed in the viewer, but is not accessible.
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Property {
    pub name: Identifier,
    pub value: Value,
    pub src_ref: SrcRef,
    pub ty: PropertyType,
}

/// Builder method
impl Property {
    pub fn new(name: impl AsRef<str>, value: impl Into<Value>, ty: PropertyType) -> Self {
        Self {
            name: Identifier::from(name.as_ref()),
            value: value.into(),
            src_ref: SrcRef::none(),
            ty,
        }
    }

    pub fn input(name: impl AsRef<str>, value: impl Into<Value>) -> Self {
        Self::new(name, value, PropertyType::Input)
    }

    pub fn output(name: impl AsRef<str>, value: impl Into<Value>) -> Self {
        Self::new(name, value, PropertyType::Output)
    }

    pub fn hidden(name: impl AsRef<str>, value: impl Into<Value>) -> Self {
        Self::new(name, value, PropertyType::Hidden)
    }

    pub fn with_src_ref(mut self, src_ref: SrcRef) -> Self {
        self.src_ref = src_ref;
        self
    }
}

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "- {name}{property_type}: {ty} = {value}",
            name = self.name,
            property_type = match self.ty {
                PropertyType::Hidden => "",
                PropertyType::Input => " [in]",
                PropertyType::Output => " [out]",
            },
            ty = self.value.ty(),
            value = self.value
        )
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct Properties {
    props: BTreeMap<Identifier, Property>,
}

impl std::fmt::Display for Properties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.iter().try_for_each(|(_, prop)| writeln!(f, "{prop}"))
    }
}

impl Properties {
    /// Creates a new, empty set of properties.
    pub fn new() -> Self {
        Self {
            props: BTreeMap::new(),
        }
    }

    pub fn inputs(inputs: Arguments) -> Self {
        let mut properties = Properties::new();

        for (name, value) in inputs.named_iter() {
            properties.set_property(Property::input(name.clone(), value.clone()));
        }

        properties
    }

    pub fn hidden_inputs(inputs: Arguments) -> Self {
        let mut properties = Properties::new();

        for (name, value) in inputs.named_iter() {
            properties.set_property(Property::hidden(name.clone(), value.clone()));
        }

        properties
    }

    /// Sets or updates a property.
    ///
    /// If the property already exists, its value, visibility, and source reference are updated.
    /// Returns a reference to the newly inserted or updated [`Property`].
    pub fn set_property(&mut self, property: impl Into<Property>) -> &Property {
        let property = property.into();
        let name = property.name.clone();
        self.props.insert(name.clone(), property);
        self.props.get(&name).expect("Property was just inserted")
    }

    /// Returns `true` if a property with the given name exists.
    pub fn contains(&self, name: impl Into<Identifier>) -> bool {
        self.props.contains_key(&name.into())
    }

    /// Returns the number of properties.
    pub fn len(&self) -> usize {
        self.props.len()
    }

    /// Returns `true` if there are no properties set.
    pub fn is_empty(&self) -> bool {
        self.props.is_empty()
    }
}

/// Property access trait.
pub trait GetProperty {
    /// Gets a reference to a property by its name.
    fn get_property(&self, name: impl AsRef<str>) -> Option<&Property>;
}

impl GetProperty for Properties {
    fn get_property(&self, name: impl AsRef<str>) -> Option<&Property> {
        self.props.get(name.as_ref())
    }
}

// --- Ergonomics Implementations ---

impl Deref for Properties {
    type Target = BTreeMap<Identifier, Property>;

    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl IntoIterator for Properties {
    type Item = (Identifier, Property);
    type IntoIter = std::collections::btree_map::IntoIter<Identifier, Property>;

    fn into_iter(self) -> Self::IntoIter {
        self.props.into_iter()
    }
}

impl<'a> IntoIterator for &'a Properties {
    type Item = (&'a Identifier, &'a Property);
    type IntoIter = std::collections::btree_map::Iter<'a, Identifier, Property>;

    fn into_iter(self) -> Self::IntoIter {
        self.props.iter()
    }
}

impl From<Arguments> for Properties {
    fn from(args: Arguments) -> Self {
        Self::inputs(args)
    }
}

impl FromIterator<Property> for Properties {
    fn from_iter<T: IntoIterator<Item = Property>>(iter: T) -> Self {
        let mut props = Properties::new();
        iter.into_iter().for_each(|property| {
            props.set_property(property);
        });
        props
    }
}
