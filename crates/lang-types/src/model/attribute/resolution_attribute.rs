// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resolution attribute.

use crate::{Length, Quantity, QuantityType, Scalar, Value, ValueError};
use serde::{Deserialize, Serialize};

/// Render resolution when rendering things e.g. to polygons or meshes.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ResolutionAttribute {
    /// Linear resolution in millimeters (Default = 0.1mm)
    Absolute(Length),

    /// Relative resolution.
    Relative(Scalar),
}

impl Default for ResolutionAttribute {
    fn default() -> Self {
        Self::Absolute(Length::mm(0.1))
    }
}

impl From<ResolutionAttribute> for Value {
    fn from(resolution_attribute: ResolutionAttribute) -> Self {
        Self::Quantity(match resolution_attribute {
            ResolutionAttribute::Absolute(linear) => Quantity::from(linear),
            ResolutionAttribute::Relative(relative) => {
                Quantity::new(relative, QuantityType::Scalar)
            }
        })
    }
}

impl TryFrom<Value> for ResolutionAttribute {
    type Error = ValueError;

    /// An absolute resolution from a length, e.g. `0.1mm`,
    /// or a relative resolution from a scalar, e.g. `200%`.
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Quantity(Quantity {
                value,
                quantity_type: QuantityType::Length,
                ..
            }) => Ok(Self::Absolute(Length(value))),
            Value::Quantity(Quantity {
                value,
                quantity_type: QuantityType::Scalar,
                ..
            }) => Ok(Self::Relative(value)),
            value => Err(ValueError::CannotConvert(
                value.to_string(),
                "Length or Scalar".into(),
            )),
        }
    }
}

impl std::fmt::Display for ResolutionAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionAttribute::Absolute(linear) => write!(f, "Linear({linear} mm)"),
            ResolutionAttribute::Relative(relative) => write!(f, "Relative({relative}%)"),
        }
    }
}
