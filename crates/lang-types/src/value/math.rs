// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Math functions for Value

use crate::{MathOps, Quantity, Ty, Value, ValueError, ValueResult};

impl MathOps for Value {
    type Error = ValueError;
    type Output = Value;

    /// Calculate the absolute value.
    fn abs(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Ok(Value::Integer(i.abs())),
            Value::Quantity(q) => q.abs(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the square root.
    fn sqrt(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Quantity::from(*i).sqrt(),
            Value::Quantity(q) => q.sqrt(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the sine (expects radians or Quantity with angle units).
    fn sin(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Quantity::from(*i).sin(),
            Value::Quantity(q) => q.sin(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the cosine (expects radians or Quantity with angle units).
    fn cos(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Quantity::from(*i).cos(),
            Value::Quantity(q) => q.cos(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the tangent (expects radians or Quantity with angle units).
    fn tan(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Quantity::from(*i).tan(),
            Value::Quantity(q) => q.tan(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    fn int(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Ok(Value::Integer(*i)),
            Value::Quantity(q) => q.int(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Calculate the sign of the value (-1, 0, or 1).
    fn signum(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Ok(Value::Integer(i.signum())),
            Value::Quantity(q) => q.signum(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Round down to the nearest integer.
    fn floor(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Ok(Value::Integer(*i)),
            Value::Quantity(q) => q.floor(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Round up to the nearest integer.
    fn ceil(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Ok(Value::Integer(*i)),
            Value::Quantity(q) => q.ceil(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Round to the nearest integer.
    fn round(&self) -> ValueResult {
        match self {
            Value::Integer(i) => Ok(Value::Integer(*i)),
            Value::Quantity(q) => q.round(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }

    /// Extract the fractional part.
    fn fract(&self) -> ValueResult {
        match self {
            Value::Integer(_) => Ok(Value::Integer(0.into())),
            Value::Quantity(q) => q.fract(),
            val => Err(ValueError::InvalidMathOperand(val.ty())),
        }
    }
}
