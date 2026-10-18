// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Type list type syntax element

use crate::ty::*;

/// List of types
pub struct TypeList(Vec<Type>);

impl TypeList {
    /// Create new type list
    pub fn new(types: Vec<Type>) -> Self {
        Self(types)
    }

    /// Check if all list items are of a common type
    pub fn common_type(&self) -> Option<Type> {
        if let Some(ty) = self.0.first()
            && self.0[1..].iter().all(|t| t == ty)
        {
            return Some(ty.clone());
        }
        None
    }
}

impl std::fmt::Debug for TypeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::fmt::Display for TypeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
