// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Matrix value type

mod math;
pub mod ops;

use crate::{ValueResult, ty::*};

use mucad_lang_base::element::BinaryOperator;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{Integer, Length, Scalar};

/// A numeric value
#[derive(Clone, Debug)]
pub struct Quantity {
    /// The numeric value of the quantity.
    pub value: Scalar,
    /// The quantity type with a base unit.
    pub quantity_type: QuantityType,
    /// The original unit of the quantity,
    pub unit: Unit,
}

impl Quantity {
    /// Create a new quantity.
    pub fn new(value: Scalar, quantity_type: QuantityType) -> Self {
        Self {
            value,
            unit: quantity_type.base_unit(),
            quantity_type,
        }
    }

    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    /// Transforms the internal value using a closure.
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Scalar) -> Scalar,
    {
        Self {
            value: f(self.value),
            quantity_type: self.quantity_type,
            unit: self.unit,
        }
    }

    /// Calculate the power of quantity.
    pub fn pow(&self, _rhs: &Quantity) -> Self {
        todo!()
    }

    /// Calculate the power of quantity and an integer.
    pub fn pow_int(&self, _rhs: &Integer) -> Self {
        todo!()
    }

    /// Compare with `rhs`, values within the rounding errors are equal, see [`approx_eq`].
    pub fn cmp(&self, op: BinaryOperator, rhs: &Self) -> ValueResult {
        let lhs = self;
        match lhs.quantity_type == rhs.quantity_type {
            true => {
                let equal = approx_eq(lhs.value, rhs.value);
                Ok(match op {
                    BinaryOperator::GreaterThan => lhs.value > rhs.value && !equal,
                    BinaryOperator::LessThan => lhs.value < rhs.value && !equal,
                    BinaryOperator::GreaterEqual => lhs.value > rhs.value || equal,
                    BinaryOperator::LessEqual => lhs.value < rhs.value || equal,
                    BinaryOperator::Equal | BinaryOperator::Near => equal,
                    BinaryOperator::NotEqual => !equal,
                    op => unreachable!("No comparison operator: {op}"),
                }
                .into())
            }
            false => {
                Err(
                    TypeError::binary_op(lhs.quantity_type.clone(), rhs.quantity_type.clone(), op)
                        .into(),
                )
            }
        }
    }
}

impl std::fmt::Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_value = self.unit.denormalize(self.value);
        let num_str = match f.precision() {
            Some(precision) => format!("{:.precision$}", display_value, precision = precision),
            // Only the decimals within the resolution, e.g. `90°` instead of `89.9999999965°`.
            None => {
                let decimals = self.unit.decimals();
                let num_str = format!("{:.decimals$}", display_value.to_num::<f64>());
                match num_str.contains('.') {
                    true => num_str
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string(),
                    false => num_str,
                }
            }
        };
        let full_str = if self.unit.is_none() {
            num_str
        } else {
            format!("{num_str}{}", self.unit)
        };

        // 3. Handle width and alignment
        if let Some(width) = f.width() {
            // Default to Right alignment for numbers unless explicitly set left/center
            let align = f.align().unwrap_or(std::fmt::Alignment::Right);

            match align {
                std::fmt::Alignment::Left => write!(f, "{:<width$}", full_str, width = width),
                std::fmt::Alignment::Right => write!(f, "{:>width$}", full_str, width = width),
                std::fmt::Alignment::Center => write!(f, "{:^width$}", full_str, width = width),
            }
        } else {
            f.write_str(&full_str)
        }
    }
}

/// Two values are equal within the rounding errors of fixed-point arithmetic,
/// e.g. `5.1 * 4 == 20.4`, although `5.1` cannot be represented exactly.
///
/// The tolerance is 64 times the resolution of a [`Scalar`] (about `1.5e-8`), or for larger
/// values `1e-8` of the larger value, because rounding errors grow with the operands.
pub fn approx_eq(a: Scalar, b: Scalar) -> bool {
    let relative = a.abs().max(b.abs()) / Scalar::from_num(100_000_000);
    let tolerance = (Scalar::DELTA * 64).max(relative);
    (a - b).abs() <= tolerance
}

impl PartialEq for Quantity {
    /// Values within the rounding errors are equal, see [`approx_eq`].
    fn eq(&self, other: &Self) -> bool {
        // 1. Ensure the types match first
        if self.quantity_type != other.quantity_type {
            return false;
        }

        approx_eq(self.value, other.value)
    }
}

impl PartialOrd for Quantity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.quantity_type == other.quantity_type {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl From<Scalar> for Quantity {
    fn from(value: Scalar) -> Self {
        Self::new(value, QuantityType::Scalar)
    }
}

impl From<Integer> for Quantity {
    fn from(value: Integer) -> Self {
        Self::new(Scalar::from(value), QuantityType::Scalar)
    }
}

impl From<Length> for Quantity {
    fn from(length: Length) -> Self {
        Self::new(length.0, QuantityType::Length)
    }
}

impl Ty for Quantity {
    fn ty(&self) -> Type {
        Type::Quantity(self.quantity_type.clone())
    }
}

impl std::hash::Hash for Quantity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        bytemuck::bytes_of(&self.value).hash(state);
        self.quantity_type.hash(state)
    }
}

impl Serialize for Quantity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            // Human-readable format (RON, JSON, etc.): Serialize as string "{value}{unit}"
            // denormalize converts standard base units back to original unit scale
            let denormalized_val = self.unit.denormalize(self.value).to_num::<f64>();
            serializer.serialize_str(&format!("{}{}", denormalized_val, self.unit))
        } else {
            // Binary format: Serialize as a 3-field tuple struct
            use serde::ser::SerializeTupleStruct;
            let mut state = serializer.serialize_tuple_struct("Quantity", 3)?;
            state.serialize_field(&self.value)?;
            state.serialize_field(&self.quantity_type)?;
            state.serialize_field(&self.unit)?;
            state.end()
        }
    }
}

impl<'de> Deserialize<'de> for Quantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(QuantityVisitor)
        } else {
            deserializer.deserialize_tuple_struct("Quantity", 3, BinaryQuantityVisitor)
        }
    }
}

/// Visitor for Human-Readable String parsing
struct QuantityVisitor;

impl<'de> serde::de::Visitor<'de> for QuantityVisitor {
    type Value = Quantity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a quantity with unit, e.g., '10.5mm'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v = v.trim();

        // Find index where unit string begins (first non-numeric/sign/decimal character)
        let split_idx = v
            .find(|c: char| {
                !c.is_numeric() && c != '.' && c != '-' && c != '+' && c != 'e' && c != 'E'
            })
            .unwrap_or(v.len());

        let (num_str, unit_str) = v.split_at(split_idx);
        let unit_str = unit_str.trim();

        if num_str.is_empty() {
            return Err(serde::de::Error::custom(format!(
                "missing numeric value in quantity string: '{v}'"
            )));
        }

        // 1. Parse number as f64 (or directly into Scalar)
        let num_f64: f64 = num_str.parse().map_err(|_| {
            serde::de::Error::custom(format!("failed to parse float from '{num_str}'"))
        })?;

        use std::str::FromStr;

        // 2. Parse Unit via FromStr (default to Unit::None if unit component is empty)
        let unit = Unit::from_str(unit_str).map_err(|_| {
            serde::de::Error::custom(format!("unknown or invalid unit '{unit_str}'"))
        })?;

        // 3. Infer QuantityType from unit
        let quantity_type = unit.quantity_type();

        // 4. Normalize value to standard base unit if needed
        let value = unit.normalize(Scalar::from_num(num_f64));

        Ok(Quantity {
            value,
            quantity_type,
            unit,
        })
    }
}

/// Visitor for Binary deserialization
struct BinaryQuantityVisitor;

impl<'de> serde::de::Visitor<'de> for BinaryQuantityVisitor {
    type Value = Quantity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple struct Quantity")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        Ok(Quantity {
            value: seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?,
            quantity_type: seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?,
            unit: seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?,
        })
    }
}
