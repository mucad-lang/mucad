// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Named tuple evaluation entity

use mucad_lang_base::Identifier;

use crate::{ty::*, value::*};

/// Tuple with positional and named values
///
/// TODO: Consider splitting this up into `Tuple` (e.g. (0.0mm, 1, 3)) and `Record` (e.g. (x: 0, y: 1, z: 2))
#[derive(Clone, Debug, Hash, Default, PartialEq, Serialize, Deserialize)]
pub struct Tuple {
    pub positional: Vec<Value>,
    pub named: Vec<(Identifier, Value)>,
}

/// Create a Tuple from items
#[macro_export]
macro_rules! tuple {
        ($($key:ident = $value:expr),*) => {
                $crate::Tuple::from_iter([$( ($crate::Identifier::no_ref(stringify!($key)), $crate::Value::from($value)) ),* ].into_iter())
    };
}

impl Tuple {
    /// Return the tuple type.
    pub fn tuple_type(&self) -> TupleType {
        TupleType {
            positional: self.positional.iter().map(|v| v.ty()).collect(),
            named: self
                .named
                .iter()
                .map(|(id, v)| (id.clone(), v.ty()))
                .collect(),
        }
    }

    pub fn get_field(&self, field: &Identifier) -> Option<&Value> {
        self.named
            .iter()
            .find_map(|(k, v)| if k == field { Some(v) } else { None })
    }

    /// Consumes `self`, removes the specified named field if present, and returns `self`.
    pub fn with_field_removed(mut self, key: &Identifier) -> Self {
        if let Some(pos) = self.named.iter().position(|(id, _)| id == key) {
            self.named.swap_remove(pos);
        }
        self
    }

    /// Checks if two tuples have matching structural shapes:
    /// 1. Same number of positional elements.
    /// 2. Same set of named field identifiers (in exact sequence).
    pub fn same_structure(&self, rhs: &Tuple) -> bool {
        // 1. Positional counts must match
        if self.positional.len() != rhs.positional.len() {
            return false;
        }

        // 2. Named field counts must match
        if self.named.len() != rhs.named.len() {
            return false;
        }

        // 3. Named identifiers must match in order
        self.named
            .iter()
            .zip(rhs.named.iter())
            .all(|((lhs_id, _), (rhs_id, _))| lhs_id == rhs_id)
    }

    /// Combine two tuples of the same structure with an operation.
    ///
    /// This function is used for `+` and `-` builtin operators.
    pub fn combine(
        self,
        rhs: Tuple,
        op: impl Fn(Value, Value) -> ValueResult,
    ) -> ValueResult<Self> {
        if self.same_structure(&rhs) {
            Ok(Tuple {
                positional: self
                    .positional
                    .into_iter()
                    .zip(rhs.positional)
                    .map(|(lhs, rhs)| op(lhs, rhs))
                    .collect::<ValueResult<Vec<_>>>()?,
                named: self
                    .named
                    .into_iter()
                    .zip(rhs.named)
                    .map(|((id, lhs), (_, rhs))| op(lhs, rhs).map(|v| (id, v)))
                    .collect::<ValueResult<Vec<_>>>()?,
            })
        } else {
            Err(ValueError::TupleTypeMismatch {
                lhs: self.ty(),
                rhs: rhs.ty(),
            })
        }
    }

    /// Apply value with an operation to a tuple.
    ///
    /// This function is used for `*` and `/` builtin operators.
    pub fn apply(
        self,
        value: Value,
        op: impl Fn(Value, Value) -> ValueResult,
    ) -> ValueResult<Self> {
        Ok(Tuple {
            positional: self
                .positional
                .into_iter()
                .map(|lhs| op(lhs, value.clone()))
                .collect::<ValueResult<Vec<_>>>()?,
            named: self
                .named
                .into_iter()
                .map(|(id, lhs)| op(lhs, value.clone()).map(|v| (id, v)))
                .collect::<ValueResult<Vec<_>>>()?,
        })
    }

    /// Transform each value in the tuple.
    pub fn transform(self, op: impl Fn(Value) -> ValueResult) -> ValueResult<Self> {
        Ok(Tuple {
            positional: self
                .positional
                .into_iter()
                .map(&op)
                .collect::<ValueResult<Vec<_>>>()?,
            named: self
                .named
                .into_iter()
                .map(|(id, v)| op(v).map(|v| (id, v)))
                .collect::<ValueResult<Vec<_>>>()?,
        })
    }

    /// Negates all positional and named values in the tuple in-place.
    /// Returns an error if any value cannot be negated.
    pub fn neg_in_place(&mut self) -> Result<(), ValueError> {
        // Negate positional values
        for val in &mut self.positional {
            *val = val.neg_value()?;
        }

        // Negate named parameter values
        for (_id, val) in &mut self.named {
            *val = val.neg_value()?;
        }

        Ok(())
    }
}

impl<T> From<std::slice::Iter<'_, (&'static str, T)>> for Tuple
where
    T: Into<Value> + Clone,
{
    fn from(iter: std::slice::Iter<'_, (&'static str, T)>) -> Self {
        let (positional, named): (Vec<_>, _) = iter
            .map(|(k, v)| (Identifier::no_ref(k), (*v).clone().into()))
            .partition(|(k, _)| k.is_empty());
        Self {
            positional: positional.into_iter().map(|(_, v)| v).collect(),
            named: named.into_iter().collect(),
        }
    }
}

impl<T> From<std::slice::Iter<'_, T>> for Tuple
where
    T: Into<Value> + Clone,
{
    fn from(iter: std::slice::Iter<'_, T>) -> Self {
        Self {
            positional: iter.map(|v| v.clone().into()).collect::<Vec<_>>(),
            named: vec![],
        }
    }
}

impl FromIterator<(Identifier, Value)> for Tuple {
    fn from_iter<T: IntoIterator<Item = (Identifier, Value)>>(iter: T) -> Self {
        let (positional, named): (Vec<_>, _) = iter
            .into_iter()
            .map(|(k, v)| (k, v.clone()))
            .partition(|(k, _)| k.is_empty());
        Self {
            positional: positional.into_iter().map(|(_, v)| v).collect(),
            named: named.into_iter().collect(),
        }
    }
}

impl From<Vec2> for Tuple {
    fn from(v: Vec2) -> Self {
        tuple!(x = v.x, y = v.y)
    }
}

impl From<Vec3> for Tuple {
    fn from(v: Vec3) -> Self {
        tuple!(x = v.x, y = v.y, z = v.z)
    }
}

impl From<Color> for Tuple {
    fn from(color: Color) -> Self {
        tuple!(r = color.r, g = color.g, b = color.b, a = color.a)
    }
}

impl std::fmt::Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.positional
                .iter()
                .map(|v| v.to_string())
                .chain(self.named.iter().map(|(id, v)| format!("{id} = {v}")))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl std::ops::Add<Tuple> for Tuple {
    type Output = ValueResult<Tuple>;

    fn add(self, rhs: Tuple) -> Self::Output {
        self.combine(rhs, |lhs, rhs| lhs.clone() + rhs.clone())
    }
}

impl std::ops::Sub<Tuple> for Tuple {
    type Output = ValueResult<Tuple>;

    fn sub(self, rhs: Tuple) -> Self::Output {
        self.combine(rhs, |lhs, rhs| lhs.clone() - rhs.clone())
    }
}

impl std::ops::Mul<Value> for Tuple {
    type Output = ValueResult<Tuple>;

    fn mul(self, rhs: Value) -> Self::Output {
        self.apply(rhs, |lhs, rhs| lhs * rhs)
    }
}

impl std::ops::Div<Value> for Tuple {
    type Output = ValueResult<Tuple>;

    fn div(self, rhs: Value) -> Self::Output {
        self.apply(rhs, |lhs, rhs| lhs / rhs)
    }
}

impl std::ops::Neg for Tuple {
    type Output = ValueResult;

    fn neg(self) -> Self::Output {
        Ok(self.transform(|value| -value)?.into())
    }
}

impl std::ops::Not for Tuple {
    type Output = ValueResult;

    fn not(self) -> Self::Output {
        Ok(self.transform(|value| !value)?.into())
    }
}

impl Ty for Tuple {
    fn ty(&self) -> Type {
        Type::Tuple(Box::new(self.tuple_type()))
    }
}
