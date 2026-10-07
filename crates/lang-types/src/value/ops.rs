// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language types and values.

use microcad_lang_base::element::BinaryOperator;

use crate::{List, Quantity, QuantityType, Scalar, Ty, Type, Unit, Value, ValueError, ValueResult};

impl Value {
    pub fn cmp(&self, op: BinaryOperator, rhs: &Self) -> ValueResult {
        match (self, rhs) {
            (Value::Quantity(lhs), Value::Quantity(rhs)) => lhs.cmp(op, rhs),
            (Value::Integer(lhs), Value::Integer(rhs)) => {
                let res = match op {
                    BinaryOperator::Equal => lhs == rhs,
                    BinaryOperator::NotEqual => lhs != rhs,
                    BinaryOperator::GreaterThan => lhs > rhs,
                    BinaryOperator::GreaterEqual => lhs >= rhs,
                    BinaryOperator::LessThan => lhs < rhs,
                    BinaryOperator::LessEqual => lhs <= rhs,
                    BinaryOperator::Near => lhs == rhs, // Integers are exact
                    _ => unreachable!(),
                };
                Ok(Value::Bool(res))
            }
            (Value::Integer(lhs), Value::Quantity(rhs)) => {
                let lhs_q = Quantity::from(*lhs);
                lhs_q.cmp(op, rhs)
            }
            (Value::Quantity(lhs), Value::Integer(rhs)) => {
                let rhs_q = Quantity::from(*rhs);
                lhs.cmp(op, &rhs_q)
            }
            (lhs, rhs) => match op {
                BinaryOperator::Equal => Ok((lhs == rhs).into()),
                BinaryOperator::NotEqual => Ok((lhs != rhs).into()),
                _ => unimplemented!(),
            },
        }
    }

    /// Helper method to perform unary negation on a single Value.
    pub fn neg_value(&self) -> Result<Value, ValueError> {
        match self {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Quantity(q) => Ok(Value::Quantity(-q.clone())),
            Value::List(a) => {
                let mut mutated = a.clone();
                std::rc::Rc::make_mut(&mut mutated).neg_in_place()?;
                Ok(Value::List(mutated))
            }
            Value::Tuple(t) => {
                let mut mutated = t.clone();
                std::rc::Rc::make_mut(&mut mutated).neg_in_place()?;
                Ok(Value::Tuple(mutated))
            }
            _ => Err(ValueError::InvalidOperator("-".into())),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = ValueResult;

    fn neg(self) -> Self::Output {
        match self {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Quantity(q) => Ok(Value::Quantity(q.neg())),
            Value::List(mut a) => {
                // Mutates in-place if refcount == 1; clones container only if shared.
                std::rc::Rc::make_mut(&mut a).neg_in_place()?;

                // `a` is already the mutated Rc<List>!
                Ok(Value::List(a))
            }
            Value::Tuple(mut t) => {
                std::rc::Rc::make_mut(&mut t).neg_in_place()?;
                Ok(Value::Tuple(t))
            }
            _ => Err(ValueError::InvalidOperator("-".into())),
        }
    }
}

impl std::ops::Not for Value {
    type Output = ValueResult;

    fn not(self) -> Self::Output {
        match self {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::List(a) => !a.as_ref().clone(), // TODO This could be optimized via applying `not` in-place
            Value::Tuple(t) => !t.as_ref().clone(), // TODO This could be optimized via applying `not` in-place
            _ => Err(ValueError::InvalidOperator("!".into())),
        }
    }
}

/// Rules for operator `+`.
impl std::ops::Add for Value {
    type Output = ValueResult;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Add two integers
            (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs + rhs)),
            // Add a quantity to an integer
            (Value::Integer(lhs), Value::Quantity(rhs)) => lhs + rhs,
            // Add an integer to a quantity
            (Value::Quantity(lhs), Value::Integer(rhs)) => lhs + rhs,
            // Add two scalars
            (Value::Quantity(lhs), Value::Quantity(rhs)) => lhs + rhs,
            // Concatenate two strings
            (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
            // Concatenate two lists
            (Value::List(lhs), Value::List(rhs)) => {
                if lhs.ty() != rhs.ty() {
                    return Err(ValueError::CannotCombineVecOfDifferentType(
                        lhs.ty(),
                        rhs.ty(),
                    ));
                }

                Ok(List::from_iter(lhs.iter().chain(rhs.iter()).cloned()).into())
            }
            // Add a value to an list.
            (Value::List(lhs), rhs) => Ok((lhs.as_ref().clone() + rhs)?), // TODO This could be optimized via applying `not` in-place
            // Add two tuples of the same type: (x = 1., y = 2.) + (x = 3., y = 4.)
            (Value::Tuple(lhs), Value::Tuple(rhs)) => {
                Ok((lhs.as_ref().clone() + rhs.as_ref().clone())?.into()) // TODO This could be optimized via applying `not` in-place
            } // TODO This could be optimized via applying `not` in-place
            (lhs, rhs) => Err(ValueError::InvalidOperator(format!("{lhs} + {rhs}"))),
        }
    }
}

/// Rules for operator `-`.
impl std::ops::Sub for Value {
    type Output = ValueResult;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Subtract two integers
            (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs - rhs)),
            // Subtract an scalar and an integer
            (Value::Quantity(lhs), Value::Integer(rhs)) => lhs - rhs,
            // Subtract an integer and a scalar
            (Value::Integer(lhs), Value::Quantity(rhs)) => lhs - rhs,
            // Subtract two numbers
            (Value::Quantity(lhs), Value::Quantity(rhs)) => lhs - rhs,
            // Subtract value to an list: `[1,2,3] - 1 = [0,1,2]`.
            (Value::List(lhs), rhs) => lhs.as_ref().clone() - rhs,
            // Subtract two tuples of the same type: (x = 1., y = 2.) - (x = 3., y = 4.)
            (Value::Tuple(lhs), Value::Tuple(rhs)) => {
                Ok((lhs.as_ref().clone() - rhs.as_ref().clone())?.into())
            } // TODO This could be optimized via applying `not` in-place

            // Boolean difference operator for models
            (lhs, rhs) => Err(ValueError::InvalidOperator(format!("{lhs} - {rhs}"))),
        }
    }
}

/// Rules for operator `*`.
impl std::ops::Mul for Value {
    type Output = ValueResult;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Multiply two integers
            (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs * rhs)),
            // Multiply an integer and a scalar, result is scalar
            (Value::Integer(lhs), Value::Quantity(rhs)) => lhs * rhs,
            // Multiply a scalar and an integer, result is scalar
            (Value::Quantity(lhs), Value::Integer(rhs)) => lhs * rhs,
            // Multiply two scalars
            (Value::Quantity(lhs), Value::Quantity(rhs)) => lhs * rhs,
            (Value::List(list), value) | (value, Value::List(list)) => {
                Ok((list.as_ref().clone() * value)?)
            }

            (Value::Tuple(tuple), value) | (value, Value::Tuple(tuple)) => {
                Ok((tuple.as_ref().clone() * value)?.into())
            }
            (lhs, rhs) => Err(ValueError::InvalidOperator(format!("{lhs} * {rhs}"))),
        }
    }
}

/// Multiply a Unit with a value. Used for unit bundling: `[1,2,3]mm`.
///
/// `[1,2,3]mm` is a shortcut for `[1,2,3] * 1mm`.
impl std::ops::Mul<Unit> for Value {
    type Output = ValueResult;

    fn mul(self, unit: Unit) -> Self::Output {
        match (self, unit.ty()) {
            (value, Type::Quantity(QuantityType::Scalar)) | (value, Type::Integer) => Ok(value),
            (Value::Integer(i), Type::Quantity(quantity_type)) => Ok(Value::Quantity(
                Quantity::new(unit.normalize(Scalar::from(i)), quantity_type),
            )),
            (Value::Quantity(quantity), Type::Quantity(quantity_type)) => {
                quantity * Quantity::new(unit.factor(), quantity_type)
            }
            (Value::List(list), Type::Quantity(quantity_type)) => Ok((list.as_ref().clone()
                * Value::Quantity(Quantity::new(unit.factor(), quantity_type)))?),
            (value, _) => Err(ValueError::CannotAddUnitToValueWithUnit(value.to_string())),
        }
    }
}

/// Rules for operator `/`.
impl std::ops::Div for Value {
    type Output = ValueResult;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Division with scalar result
            (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Quantity(
                (Scalar::from(lhs) / Scalar::from(rhs)).into(),
            )),
            (Value::Quantity(lhs), Value::Integer(rhs)) => lhs / rhs,
            (Value::Integer(lhs), Value::Quantity(rhs)) => lhs / rhs,
            (Value::Quantity(lhs), Value::Quantity(rhs)) => lhs / rhs,
            (Value::List(list), value) => Ok((list.as_ref().clone() / value)?),
            (Value::Tuple(tuple), value) => Ok((tuple.as_ref().clone() / value)?.into()),
            (lhs, rhs) => Err(ValueError::InvalidOperator(format!("{lhs} / {rhs}"))),
        }
    }
}

/// Rules for operator `|`` (union).
impl std::ops::BitOr for Value {
    type Output = ValueResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs | rhs)),
            (lhs, rhs) => Err(ValueError::InvalidOperator(format!("{lhs} | {rhs}"))),
        }
    }
}

/// Rules for operator `&` (intersection).
impl std::ops::BitAnd for Value {
    type Output = ValueResult;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs & rhs)),
            (lhs, rhs) => Err(ValueError::InvalidOperator(format!("{lhs} & {rhs}"))),
        }
    }
}
