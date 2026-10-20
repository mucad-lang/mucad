// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model tree module

pub mod attribute;
pub mod creator;
pub mod element;
pub mod output_type;

mod prop;
pub use prop::{GetProperty, Properties, Property, PropertyType};

mod tree;

pub use tree::{
    Arena, BuildModelTreeError, ModelNodeId, ModelTree, ModelTreeBuilder, ModelTreeBuilderMut,
    Node, NodeExt, NodeMut, NodeRef,
};

use mucad_lang_base::{BuiltinId, DisplayWithCtx, HashId, Identifier, LookUpName};
use serde::{Deserialize, Serialize};

pub use attribute::{Attribute, AttributeAccess, Attributes};

pub use creator::Creator;
pub use element::{BooleanOp, BuiltinWorkpiece, Element, Workpiece};
pub use output_type::ModelType;

use crate::{
    Arguments, Mat4, Ty, Type, Value, ValueError, ValueResult, math::AffineTransform,
    model::attribute::ResolutionAttribute,
};

#[derive(Debug, Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Model {
    /// An optional name
    pub name: Option<Identifier>,

    /// Model attributes `#`
    pub attr: Attributes,

    /// Model properties
    pub properties: Properties,

    /// Model element
    pub element: Element,

    /// The hash of the element, properties, attributes and children,
    /// see [`ModelTree::compute_hashes`].
    pub hash_id: HashId,

    /// The call that created this model
    pub creator: Option<Creator>,
}

/// Builder functions
impl Model {
    pub fn new(element: impl Into<Element>) -> Self {
        Self {
            element: element.into(),
            ..Default::default()
        }
    }

    pub fn with_name(mut self, name: impl Into<Identifier>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_attr(mut self, attr: impl Into<Attributes>) -> Self {
        self.attr = attr.into();
        self
    }

    pub fn with_op_properties(mut self, args: Arguments) -> Self {
        let args = args.with_field_removed("self");
        self.properties = Properties::hidden_inputs(args);
        self
    }

    pub fn with_properties(mut self, properties: impl Into<Properties>) -> Self {
        self.properties = properties.into();
        self
    }
}

/// Accessor functions
impl Model {
    pub fn builtin_id(&self) -> Option<BuiltinId> {
        match &self.element {
            Element::BuiltinWorkpiece(builtin_workpiece) => builtin_workpiece.builtin_id(),
            _ => None,
        }
    }

    pub fn output_type(&self) -> ModelType {
        self.element.output_type()
    }

    pub fn get_property(&self, name: impl AsRef<str>) -> Option<&Property> {
        self.properties.get_property(name.as_ref())
    }

    pub fn get_property_value(&self, name: impl AsRef<str>) -> Value {
        self.get_property(name)
            .map(|property| property.value.clone())
            .unwrap_or_default()
    }

    pub fn get_property_as<T: TryFrom<Value, Error = ValueError>>(
        &self,
        name: impl AsRef<str>,
    ) -> ValueResult<T> {
        T::try_from(self.get_property_value(name))
    }

    pub fn local_matrix(&self) -> Mat4 {
        use crate::math::{FromFloat, Mat4F};
        use cgmath::SquareMatrix;

        match &self.element {
            Element::BuiltinWorkpiece(BuiltinWorkpiece::AffineTransform(t)) => t.matrix(),
            _ => Mat4::from_float(Mat4F::identity()),
        }
    }

    /// Get resolution of this model from attributes.
    pub fn resolution(&self) -> Option<&ResolutionAttribute> {
        self.attr.resolution()
    }
}

impl AttributeAccess for Model {
    fn get_attribute(&self, name: impl AsRef<str>) -> Option<Value> {
        self.attr.get_attribute(name)
    }
}

impl From<Element> for Model {
    fn from(element: Element) -> Self {
        Self::new(element)
    }
}

impl From<AffineTransform> for Model {
    fn from(transform: AffineTransform) -> Self {
        Self::new(Element::BuiltinWorkpiece(
            BuiltinWorkpiece::AffineTransform(Box::new(transform)),
        ))
    }
}

impl From<BooleanOp> for Model {
    fn from(op: BooleanOp) -> Self {
        Self::new(Element::BuiltinWorkpiece(BuiltinWorkpiece::BooleanOp(op)))
    }
}

impl<Ctx: LookUpName> DisplayWithCtx<Ctx> for Model {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{name}: ")?;
        }

        self.element.fmt_with_ctx(f, ctx)?;
        writeln!(f)?;
        self.attr
            .iter()
            .try_for_each(|attr| writeln!(f, "    {attr}"))?;
        self.properties
            .iter()
            .try_for_each(|(_, prop)| writeln!(f, "    {prop}"))
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_ctx(f, &mucad_lang_base::DefaultContext)
    }
}

impl Ty for Model {
    fn ty(&self) -> crate::Type {
        Type::Model(self.output_type())
    }
}
