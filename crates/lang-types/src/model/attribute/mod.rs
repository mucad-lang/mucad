// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model attributes.

mod export_command;
mod layer;
mod resolution_attribute;

use crate::{Color, Value};
pub use export_command::ExportCommand;
pub use layer::Layer;
pub use resolution_attribute::ResolutionAttribute;

use mucad_lang_base::Identifier;
use serde::{Deserialize, Serialize};

/// An attribute for a model.
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum Attribute {
    /// Color attribute: `color = "red"`
    Color(Color),
    /// Render resolution attribute: `resolution = 200%`.
    Resolution(ResolutionAttribute),
    /// Export command: `export = "test.svg"`.
    Export(ExportCommand),
}

impl Attribute {
    /// Return an id for the attribute.
    fn id(&self) -> Identifier {
        match &self {
            Attribute::Color(_) => Identifier::no_ref("color"),
            Attribute::Resolution(_) => Identifier::no_ref("resolution"),
            Attribute::Export(_) => Identifier::no_ref("export"),
        }
    }

    /// If this method returns true, the attribute can only be set once.
    pub fn is_unique(&self) -> bool {
        matches!(self, Attribute::Color(_) | Attribute::Resolution(_))
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "# {id} = {value}",
            id = self.id(),
            value = match &self {
                Attribute::Color(color) => format!("{color}"),
                Attribute::Resolution(resolution) => format!("{resolution}"),
                Attribute::Export(export) => format!("{export}"),
            }
        )
    }
}

#[derive(Clone, Default, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub struct Attributes(pub Vec<Attribute>);

impl Attributes {
    /// Returns the first `ResolutionAttribute` if present.
    pub fn resolution(&self) -> Option<&ResolutionAttribute> {
        self.0.iter().find_map(|attr| match attr {
            Attribute::Resolution(res) => Some(res),
            _ => None,
        })
    }

    /// Add an attribute, which replaces a unique attribute of the same kind, e.g. a color.
    pub fn set(&mut self, attribute: Attribute) {
        if attribute.is_unique() {
            self.0.retain(|attr| attr.id() != attribute.id());
        }
        self.0.push(attribute);
    }

    /// Iterator over properties
    pub fn iter(&self) -> impl Iterator<Item = &Attribute> {
        self.0.iter()
    }
}

pub trait AttributeAccess {
    /// Gets the value of the first attribute matching `name`.
    ///
    /// Returns `None` if no attribute with the given identifier exists.
    fn get_attribute(&self, name: impl AsRef<str>) -> Option<Value>;
}

impl AttributeAccess for Attributes {
    fn get_attribute(&self, name: impl AsRef<str>) -> Option<Value> {
        let key = name.as_ref();

        self.0.iter().find_map(|attr| {
            if &attr.id() == key {
                let value = match attr {
                    Attribute::Color(color) => Value::from(*color),
                    Attribute::Resolution(res) => Value::from(res.clone()),
                    Attribute::Export(export) => Value::from(export.clone()),
                };
                Some(value)
            } else {
                None
            }
        })
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|attr| attr.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
