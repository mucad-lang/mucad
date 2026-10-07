// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed list of values evaluation entity

use crate::{ty::*, value::*};
use derive_more::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

/// Collection of values of the same type.
#[derive(Clone, Default, Debug, Deref, Hash, PartialEq, DerefMut, Serialize, Deserialize)]
pub struct List {
    /// List of values
    #[deref]
    #[deref_mut]
    items: ValueList,
    /// Element type.
    ty: Type,
}

impl List {
    /// Create new list from `ValueList`.
    pub fn new(items: ValueList, ty: Type) -> Self {
        Self { items, ty }
    }

    pub fn from_values(items: ValueList) -> Self {
        let ty = items.types().common_type().unwrap_or_default();
        Self { items, ty }
    }

    /// Negates all elements in the list in-place.
    /// Returns an error if any element cannot be negated (e.g., strings or booleans).
    pub fn neg_in_place(&mut self) -> Result<(), ValueError> {
        for val in &mut self.items.iter_mut() {
            // Replaces each element with its negated version
            *val = val.neg_value()?;
        }
        Ok(())
    }
}

/// All builtin methods and builtin functions.
///
/// `len` and `contains` are already implemented and accessible via `impl Deref`.
impl List {
    /// Get the first element, or None
    pub fn first(&self) -> Value {
        self.items.first().cloned().unwrap_or_default()
    }

    /// Get the last element, or None
    pub fn last(&self) -> Value {
        self.items.last().cloned().unwrap_or_default()
    }

    /// Get first `n` elements
    pub fn head(&self, n: Integer) -> List {
        List::new(
            self.items.iter().take(n.to_num()).cloned().collect(),
            self.ty.clone(),
        )
    }

    /// Get all elements but the first `n`
    pub fn tail(&self, n: Integer) -> List {
        List::new(
            self.items.iter().skip(n.to_num()).cloned().collect(),
            self.ty.clone(),
        )
    }

    /// Return a reversed version of the list.
    pub fn rev(&self) -> List {
        List::new(self.items.iter().rev().cloned().collect(), self.ty.clone())
    }

    /// Return a sorted version of this list.
    ///
    /// Only primitive types (quantities, integers, bools and string) can be sorted.
    pub fn sorted(&self) -> List {
        let mut items = self.items.clone();
        match self.ty {
            Type::Integer | Type::Quantity(..) | Type::String | Type::Bool => {
                items.sort_by(|a, b| {
                    assert_eq!(a.ty(), b.ty());
                    match (a, b) {
                        (Value::Quantity(a), Value::Quantity(b)) => a.value.cmp(&b.value),
                        (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
                        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
                        (Value::String(a), Value::String(b)) => a.cmp(b),
                        _ => unreachable!(),
                    }
                })
            }
            _ => {}
        };

        List::new(items, self.ty.clone())
    }

    /// Check if all items are equal.
    pub fn all_equal(&self) -> bool {
        match self.first() {
            Value::None => true,
            value => self[1..].iter().all(|x| *x == value),
        }
    }

    /// Check if all items are sorted in ascending order.
    pub fn is_ascending(&self) -> bool {
        self.as_slice().windows(2).all(|w| w[0] <= w[1])
    }

    /// Check if all items are sorted in descending order.
    pub fn is_descending(&self) -> bool {
        self.as_slice().windows(2).all(|w| w[0] >= w[1])
    }
}

impl IntoIterator for List {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl TryFrom<ValueList> for List {
    type Error = ValueError;
    fn try_from(items: ValueList) -> ValueResult<List> {
        match items.types().common_type() {
            Some(ty) => Ok(List::new(items, ty)),
            None => Err(ValueError::CommonTypeExpected),
        }
    }
}

impl FromIterator<Value> for List {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        let items: ValueList = iter.into_iter().collect();
        let ty = items.types().common_type().expect("Common type");
        Self { ty, items }
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{items}]",
            items = self
                .items
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl crate::ty::Ty for List {
    fn ty(&self) -> Type {
        Type::List(Box::new(self.ty.clone()))
    }
}

/// + operator. Adds a value to a list, e.g.: `[1,2] + 1 == [2,3]`.
impl std::ops::Add<Value> for List {
    type Output = ValueResult;

    fn add(self, rhs: Value) -> Self::Output {
        if self.ty.is_compatible_to(&rhs.ty()) {
            Ok(Value::List(Rc::new(Self::from_values(ValueList::new(
                self.items
                    .iter()
                    .map(|value| value.clone() + rhs.clone())
                    .collect::<Result<Vec<_>, _>>()?,
            )))))
        } else {
            Err(ValueError::InvalidOperator("+".into()))
        }
    }
}

/// - operator. Subtracts a value from an list, e.g.: `[1,2] - 1 == [0,1]`.
impl std::ops::Sub<Value> for List {
    type Output = ValueResult;

    fn sub(self, rhs: Value) -> Self::Output {
        if self.ty.is_compatible_to(&rhs.ty()) {
            Ok(Self::from_iter(
                self.items
                    .iter()
                    .map(|value| value.clone() - rhs.clone())
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .into())
        } else {
            Err(ValueError::InvalidOperator("-".into()))
        }
    }
}

/// * operator. Multiply a value from an list, e.g.: `[1,2] * 2 == [2,4]`.
impl std::ops::Mul<Value> for List {
    type Output = ValueResult;

    fn mul(self, rhs: Value) -> Self::Output {
        match self.ty {
            // List * Scalar or List * Integer
            Type::Quantity(_) | Type::Integer => {
                let values = self
                    .iter()
                    .map(|value| value.clone() * rhs.clone())
                    .collect::<Result<Vec<_>, _>>()?;

                // `from_values` infers `Type` from the actual multiplied element values
                Ok(List::from_values(ValueList::new(values)).into())
            }
            _ => Err(ValueError::InvalidOperator("*".into())),
        }
    }
}

/// / operator. Divide an list by value, e.g.: `[2,4] / 2 == [1,2]`.
impl std::ops::Div<Value> for List {
    type Output = ValueResult;

    fn div(self, rhs: Value) -> Self::Output {
        // 1. Evaluate element-wise division by cloning `rhs` per element (or referencing)
        let mut divided_values = Vec::with_capacity(self.len());
        for value in self.iter() {
            // Evaluates `value / rhs`
            divided_values.push((value.clone() / rhs.clone())?);
        }

        let values = ValueList::new(divided_values);

        // 2. Determine resulting List Type based on element type division
        // An List divided by a Scalar preserves an List structure with transformed element types
        match (&self.ty, rhs.ty()) {
            (Type::Integer, Type::Integer) => Ok(List::new(values, Type::Integer).into()),
            (Type::Quantity(_), _) => Ok(List::from_values(values).into()),
            _ => Err(ValueError::InvalidOperator("/".into())),
        }
    }
}

impl std::ops::Neg for List {
    type Output = ValueResult;

    fn neg(self) -> Self::Output {
        let items = List::from_values(ValueList::new(
            self.iter()
                .map(|value| -value.clone())
                .collect::<Result<Vec<_>, _>>()?,
        ));
        Ok(items.into())
    }
}

impl std::ops::Not for List {
    type Output = ValueResult;

    fn not(self) -> Self::Output {
        let items = List::from_values(ValueList::new(
            self.iter()
                .map(|value| !value.clone())
                .collect::<Result<Vec<_>, _>>()?,
        ));
        Ok(items.into())
    }
}

#[macro_export]
macro_rules! list {
        ($($value:expr),*) => {
                $crate::List::from_iter([$( $crate::Value::from($value)),* ].into_iter())
        }
}
