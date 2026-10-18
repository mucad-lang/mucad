// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tuple type syntax element

use crate::{BinaryOperator, Type, TypeError, TypeResult};
use mucad_lang_base::Identifier;

use serde::{Deserialize, Serialize};

/// (Partially named) tuple (e.g. `(Integer, m: Scalar, n: String)`)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TupleType {
    /// Ordered positional elements: (val0, val1, val2, ...)
    pub positional: Vec<Type>,

    /// Ordered named elements: (a = val_a, b = val_b, ...)
    pub named: Vec<(Identifier, Type)>,
}

impl TupleType {
    /// Create new Vec2 type.
    pub fn vec2() -> Self {
        [("x", Type::scalar()), ("y", Type::scalar())]
            .into_iter()
            .collect()
    }

    /// Create new Vec3 type.
    pub fn vec3() -> Self {
        [
            ("x", Type::scalar()),
            ("y", Type::scalar()),
            ("z", Type::scalar()),
        ]
        .into_iter()
        .collect()
    }

    /// Create new Color type.
    pub fn color() -> Self {
        [
            ("r", Type::scalar()),
            ("g", Type::scalar()),
            ("b", Type::scalar()),
            ("a", Type::scalar()),
        ]
        .into_iter()
        .collect()
    }

    /// Create new Size2 type.
    pub fn size2() -> Self {
        [("width", Type::length()), ("height", Type::length())]
            .into_iter()
            .collect()
    }

    /// Match tuples by id.
    pub fn matches_multiplicity(&self, params: &TupleType) -> bool {
        if self == params {
            true
        } else if self.positional.is_empty()
            && params.positional.is_empty()
            && self.named.len() == params.named.len()
        {
            self.named.iter().all(|arg| {
                if let Some((_, ty)) = params.named.iter().find(|(id, _)| id == &arg.0) {
                    arg.1 == *ty || arg.1.is_list_of(ty)
                } else {
                    false
                }
            })
        } else {
            false
        }
    }

    /// Test if the named tuple has exactly all the given keys
    fn matches_keys(&self, keys: &[&str]) -> bool {
        if !self.positional.is_empty() || self.named.len() != keys.len() {
            return false;
        }
        keys.iter().all(|k| {
            self.named
                .iter()
                .map(|(id, _)| id)
                .any(|id| id == &Identifier::no_ref(k))
        })
    }

    /// Checks if the named tuple type only holds scalar values.
    fn is_scalar_only(&self) -> bool {
        self.common_type().is_some_and(|ty| *ty == Type::scalar())
    }

    /// Checks if the named tuple type only holds length values.
    fn is_length_only(&self) -> bool {
        self.common_type().is_some_and(|ty| *ty == Type::length())
    }

    /// Test if all fields have a common type.
    pub(crate) fn common_type(&self) -> Option<&Type> {
        let mut iter = self
            .positional
            .iter()
            .chain(self.named.iter().map(|(_, ty)| ty));
        if let Some(first) = iter.next()
            && iter.all(|x| x == first)
        {
            Some(first)
        } else {
            None
        }
    }

    /// Check if the named tuple is a [`Color`].
    pub(crate) fn is_color(&self) -> bool {
        self.is_scalar_only() && self.matches_keys(&["r", "g", "b", "a"])
    }

    /// Check if the named tuple is a [`Vec2`].
    pub(crate) fn is_vec2(&self) -> bool {
        self.is_scalar_only() && self.matches_keys(&["x", "y"])
    }

    /// Check if the named tuple is a [`Vec3`].
    pub(crate) fn is_vec3(&self) -> bool {
        self.is_scalar_only() && self.matches_keys(&["x", "y", "z"])
    }

    /// Check if the named tuple is a [`Size2`]
    pub(crate) fn is_size2(&self) -> bool {
        self.is_length_only() && self.matches_keys(&["width", "height"])
    }
}

impl std::hash::Hash for TupleType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.positional.iter().for_each(|ty| ty.hash(state));
        self.named.iter().for_each(|(id, ty)| {
            id.hash(state);
            ty.hash(state)
        });
    }
}

impl FromIterator<(Identifier, Type)> for TupleType {
    fn from_iter<T: IntoIterator<Item = (Identifier, Type)>>(iter: T) -> Self {
        let (positional, named): (Vec<(_, _)>, Vec<(_, _)>) =
            iter.into_iter().partition(|(id, _)| id.is_empty());
        Self {
            positional: positional.into_iter().map(|(_, ty)| ty).collect(),
            named,
        }
    }
}

impl<'a> FromIterator<(&'a str, Type)> for TupleType {
    fn from_iter<T: IntoIterator<Item = (&'a str, Type)>>(iter: T) -> Self {
        let (positional, named): (Vec<(_, _)>, Vec<(_, _)>) = iter
            .into_iter()
            .map(|(id, ty)| (Identifier::no_ref(id), ty))
            .partition(|(id, _)| id.is_empty());
        Self {
            positional: positional.into_iter().map(|(_, ty)| ty).collect(),
            named,
        }
    }
}

impl std::fmt::Display for TupleType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_color() {
            return write!(f, "Color");
        }
        if self.is_vec2() {
            return write!(f, "Vec2");
        }
        if self.is_vec3() {
            return write!(f, "Vec3");
        }
        if self.is_size2() {
            return write!(f, "Size2");
        }

        write!(f, "({})", {
            let mut types = self
                .positional
                .iter()
                .map(|ty| ty.to_string())
                .chain(self.named.iter().map(|(id, ty)| format!("{id}: {ty}")))
                .collect::<Vec<_>>();

            types.sort();
            types.join(", ")
        })
    }
}

impl std::ops::Neg for TupleType {
    type Output = TypeResult;

    fn neg(self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Not for TupleType {
    type Output = TypeResult;

    fn not(self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Add for TupleType {
    type Output = TypeResult;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self;
        if lhs.matches_multiplicity(&rhs) {
            Ok(Type::from(Box::new(lhs)))
        } else {
            Err(TypeError::binary_op(lhs, rhs, BinaryOperator::Add))
        }
    }
}

impl std::ops::Sub for TupleType {
    type Output = TypeResult;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self;
        if lhs.matches_multiplicity(&rhs) {
            Ok(Type::from(Box::new(lhs)))
        } else {
            Err(TypeError::binary_op(lhs, rhs, BinaryOperator::Subtract))
        }
    }
}
