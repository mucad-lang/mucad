// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Values as literals in code, e.g. to change them in an editor.

use crate::{Integer, Quantity, Scalar, Value};

impl Value {
    /// The value increased by `step` in its unit, e.g. `11mm` for `10mm` and a step of `1`,
    /// `None` for other values than quantities, integers and bools, or on an overflow.
    ///
    /// An integer changes by whole steps only, a bool is toggled by any step.
    pub fn step(&self, step: Scalar) -> Option<Value> {
        match self {
            Value::Bool(value) => Some(Value::Bool(!value)),
            Value::Integer(value) if step.frac() == Scalar::ZERO => Some(Value::Integer(
                value.checked_add(step.checked_to_num::<Integer>()?)?,
            )),
            Value::Quantity(quantity) => {
                let unit = quantity.unit;
                let shown = unit.denormalize(quantity.value).checked_add(step)?;
                Some(Value::Quantity(Quantity {
                    value: unit.normalize(shown),
                    ..quantity.clone()
                }))
            }
            _ => None,
        }
    }

    /// The value as literal in code, e.g. `10mm`, `-3` or `true`, see [`Literal`], `None` for
    /// other values than quantities, integers and bools.
    pub fn literal(&self) -> Option<Literal<'_>> {
        matches!(
            self,
            Value::Quantity(_) | Value::Integer(_) | Value::Bool(_)
        )
        .then_some(Literal(self))
    }
}

/// A quantity, integer or bool value, which is displayed as literal in code, see
/// [`Value::literal`].
///
/// Unlike the value, a quantity without unit keeps a decimal point, e.g. `10.0`, so that it is
/// no integer.
#[derive(Clone, Copy, Debug)]
pub struct Literal<'a>(&'a Value);

impl std::fmt::Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Quantity(quantity) => {
                let literal = quantity.to_string();
                match quantity.unit.is_none() && !literal.contains(['.', 'e', 'E']) {
                    true => write!(f, "{literal}.0"),
                    false => write!(f, "{literal}"),
                }
            }
            value => write!(f, "{value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Unit;

    /// A quantity of `value` in `unit`, e.g. `10mm`.
    fn quantity(value: f64, unit: Unit) -> Value {
        Value::Quantity(Quantity {
            value: unit.normalize(Scalar::from_num(value)),
            quantity_type: unit.quantity_type(),
            unit,
        })
    }

    /// Quantities are stepped in their units, integers by whole steps only, bools are toggled.
    #[test]
    fn step() {
        let step = |value: &Value, step: f64| {
            value
                .step(Scalar::from_num(step))
                .and_then(|value| value.literal().map(|literal| literal.to_string()))
        };
        for (value, step_size, stepped) in [
            (quantity(10.0, Unit::Millimeter), 0.1, Some("10.1mm")),
            (quantity(10.0, Unit::Millimeter), -11.0, Some("-1mm")),
            (quantity(-1.5, Unit::Millimeter), 0.1, Some("-1.4mm")),
            (quantity(2.5, Unit::Centimeter), 0.1, Some("2.6cm")),
            (quantity(45.0, Unit::DegS), 1.0, Some("46°")),
            (quantity(50.0, Unit::Percent), 10.0, Some("60%")),
            (quantity(10.0, Unit::None), 1.0, Some("11.0")),
            (quantity(-10.5, Unit::None), 0.5, Some("-10.0")),
            (Value::Integer(Integer::from_num(3)), -10.0, Some("-7")),
            (Value::Integer(Integer::from_num(3)), 0.1, None),
            (Value::Bool(true), 0.1, Some("false")),
            (Value::String("a".into()), 1.0, None),
            (Value::None, 1.0, None),
        ] {
            assert_eq!(
                step(&value, step_size).as_deref(),
                stepped,
                "{value} {step_size}"
            );
        }

        // Steps of 0.1 are rounded in the literal.
        let mut value = quantity(0.0, Unit::Millimeter);
        for _ in 0..30 {
            value = value.step(Scalar::from_num(0.1)).expect("A quantity");
        }
        assert_eq!(
            value
                .literal()
                .map(|literal| literal.to_string())
                .as_deref(),
            Some("3mm")
        );
    }

    /// Quantities, integers and bools are displayed as literals, other values have no literal.
    #[test]
    fn literal() {
        let literal = |value: &Value| value.literal().map(|literal| literal.to_string());
        assert_eq!(
            literal(&quantity(10.0, Unit::Millimeter)).as_deref(),
            Some("10mm")
        );
        assert_eq!(
            literal(&quantity(-10.0, Unit::None)).as_deref(),
            Some("-10.0")
        );
        assert_eq!(literal(&quantity(1.5, Unit::None)).as_deref(), Some("1.5"));
        assert_eq!(
            literal(&Value::Integer(Integer::from_num(-3))).as_deref(),
            Some("-3")
        );
        assert_eq!(literal(&Value::Bool(false)).as_deref(), Some("false"));
        assert_eq!(literal(&Value::String("a".into())), None);
    }
}
