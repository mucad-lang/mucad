// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model tree builder

#[derive(Error, Debug, Diagnostic)]
pub enum BuildModelTreeError {
    #[error("Property already exists: {name}")]
    PropertyAlreadyExists {
        name: String,
        #[label]
        src_ref: SrcRef,
    },
}

use microcad_lang_base::SrcRef;
use miette::Diagnostic;
use thiserror::Error;

use crate::{
    Model, ModelTree,
    model::{Attributes, Element, Properties, Property, prop::GetProperty},
};

#[derive(Debug)]
pub struct ModelTreeBuilder {
    element: Element,
    attributes: Attributes,
    properties: Properties,
    children: Vec<ModelTree>,
}

impl ModelTreeBuilder {
    /// Create a new model builter
    pub fn new(element: impl Into<Element>) -> Self {
        Self {
            element: element.into(),
            attributes: Default::default(),
            properties: Default::default(),
            children: Default::default(),
        }
    }

    pub fn add_attributes(&mut self, attributes: Attributes) {
        self.attributes = attributes;
    }

    pub fn add_model_properties(&mut self, properties: impl IntoIterator<Item = Property>) {
        properties
            .into_iter()
            .for_each(|property| self.add_model_property(property));
    }

    pub fn add_model_property(&mut self, property: Property) {
        self.properties.set_property(property);
    }

    pub fn add_model_child(&mut self, child: impl Into<ModelTree>) {
        self.children.push(child.into());
    }

    pub fn build(&mut self) -> ModelTree {
        let model = Model::new(std::mem::take(&mut self.element))
            .with_attr(std::mem::take(&mut self.attributes))
            .with_properties(std::mem::take(&mut self.properties));

        let mut model_tree = ModelTree::new(model);
        let children = std::mem::take(&mut self.children);
        children.into_iter().for_each(|child| {
            model_tree.append(child);
        });

        model_tree
    }
}

/// Trait to add builder methods to build up a model
pub trait ModelTreeBuilderMut {
    fn model_tree_builder_mut(&mut self) -> &mut ModelTreeBuilder;
}

impl GetProperty for ModelTreeBuilder {
    fn get_property(&self, name: impl AsRef<str>) -> Option<&Property> {
        self.properties.get_property(name)
    }
}
