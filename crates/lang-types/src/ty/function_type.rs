// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A type signature for any callable (builtins, functions, workbenches).

use mucad_lang_base::Identifier;
use serde::{Deserialize, Serialize};

use crate::Type;

pub type CallParameter = (Identifier, Type);

/// A list of call parameters
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallParameters(pub Vec<CallParameter>);

impl CallParameters {
    pub fn sorted(&self) -> Self {
        let mut parameters = self.0.clone();
        parameters.sort_by_key(|(id, _)| id.clone());
        Self(parameters)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CallParameter> {
        self.0.iter()
    }

    /// Check if a signature matches another.
    ///
    /// Two signature match, if the have the same identifiers and all types match.
    pub fn matches(&self, other: &CallParameters) -> bool {
        let lhs_sorted = self.sorted();
        let rhs_sorted = other.sorted();

        lhs_sorted.len() == rhs_sorted.len()
            && lhs_sorted
                .iter()
                .zip(&rhs_sorted.0)
                .all(|((lhs_id, lhs_ty), (rhs_id, rhs_ty))| {
                    lhs_id == rhs_id && lhs_ty.matches(rhs_ty)
                })
    }

    pub fn matches_replication(&self, other: &CallParameters) -> bool {
        let lhs_sorted = self.sorted();
        let rhs_sorted = other.sorted();

        lhs_sorted.len() == rhs_sorted.len()
            && lhs_sorted
                .iter()
                .zip(&rhs_sorted.0)
                .all(|((lhs_id, lhs_ty), (rhs_id, rhs_ty))| {
                    lhs_id == rhs_id && (lhs_ty.matches(rhs_ty) || lhs_ty.is_list_of(rhs_ty))
                })
    }
}

impl From<Vec<CallParameter>> for CallParameters {
    fn from(parameters: Vec<CallParameter>) -> Self {
        Self(parameters)
    }
}

/// A type signature for any callable (builtins, functions, workbenches).
///
/// Each parameter in a call has type. The parameter list is optional.
/// If it is `None`, we have a variadic call signature.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallSignature {
    pub parameters: Option<CallParameters>,
}

impl CallSignature {
    /// Create a new signature with a parameter type list.
    pub fn new(parameters: impl Into<CallParameters>) -> Self {
        Self {
            parameters: Some(parameters.into()),
        }
    }

    pub fn variadic() -> Self {
        Self { parameters: None }
    }

    /// Returns true if the call type can accept any arguments.
    pub fn is_variadic(&self) -> bool {
        self.parameters.is_none()
    }

    /// Check if two call signatures match
    pub fn matches(&self, other: &CallSignature) -> bool {
        match (&self.parameters, &other.parameters) {
            // A variadic signature cannot metch a signature with specified signature parameters.
            (None, Some(_)) => false,
            // A variadic can accept any function
            (Some(lhs), Some(rhs)) if lhs.matches(rhs) => true,
            (Some(lhs), Some(rhs)) if !lhs.matches(rhs) => false,
            _ => true,
        }
    }

    pub fn matches_replication(&self, other: &CallSignature) -> bool {
        match (&self.parameters, &other.parameters) {
            // A variadic signature cannot metch a signature with specified signature parameters.
            (None, Some(_)) => false,
            // A variadic can accept any function
            (Some(lhs), Some(rhs)) if lhs.matches_replication(rhs) => true,
            (Some(lhs), Some(rhs)) if !lhs.matches_replication(rhs) => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for CallSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            match self.is_variadic() {
                true => String::from("*"),
                false => {
                    self.parameters
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|(id, ty)| format!("{id}: {ty}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            }
        )
    }
}

/// A type signature for any callable with return type (builtins, functions, workbenches).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionType {
    /// Parameters, sorted by id.
    /// The signature is variadic of this list is none.
    pub call_ty: CallSignature,

    /// Optional return value.
    /// If this value is None, the callable is not expected to return anything.
    pub return_ty: Option<Box<Type>>,

    /// The `is_impure` flag tells if this function has side effects.
    ///
    /// This is used for functions like `print` and `assert`, so that are not optimized away.
    pub is_impure: bool,
}

/// Builder functions.
impl FunctionType {
    /// Create a new function type with a signature and return type.
    pub fn new(call_ty: impl Into<CallSignature>, return_ty: Option<Type>) -> Self {
        Self {
            call_ty: call_ty.into(),
            return_ty: return_ty.map(Box::new),
            is_impure: false,
        }
    }

    /// Create a variadic function type with optional return type.
    pub fn variadic(return_ty: Option<Type>) -> Self {
        Self {
            call_ty: CallSignature::variadic(),
            return_ty: return_ty.map(Box::new),
            is_impure: false,
        }
    }

    /// Mark this function as impure.
    pub fn impure(mut self) -> Self {
        self.is_impure = true;
        self
    }
}

impl FunctionType {
    pub fn is_variadic(&self) -> bool {
        self.call_ty.is_variadic()
    }

    /// Check if a function call `(...) -> Type` matches `(...) -> Type`
    pub fn matches(&self, other: &FunctionType) -> bool {
        self.call_ty.matches(&other.call_ty) && self.match_return_type(other)
    }

    pub fn matches_replication(&self, other: &FunctionType) -> bool {
        self.call_ty.matches_replication(&other.call_ty) && self.match_return_type(other)
    }

    pub fn match_return_type(&self, other: &FunctionType) -> bool {
        match (&self.return_ty, &other.return_ty) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(lhs), Some(rhs)) => lhs.matches(rhs),
        }
    }
}

impl std::fmt::Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.call_ty)?;

        match &self.return_ty {
            Some(ty) => write!(f, " -> {ty}"),
            None => Ok(()),
        }
    }
}

#[macro_export]
macro_rules! call_signature {
    // 1. Variadic call signature
    (*) => {
        $crate::CallSignature::variadic()
    };

    // 2. Named parameters without return type: function_type!((a: TypeA, b: TypeB))
    ($( $param:ident : $ty:expr ),* $(,)?) => {
        $crate::CallSignature::new(vec![
            $(
                ($crate::Identifier::from(stringify!($param)), $ty)
            ),*
        ])
    };
}

#[macro_export]
macro_rules! function_type {
    // 1. Variadic with return type: function_type!(* -> ReturnTy)
    ((*) -> $ret:expr) => {
        $crate::FunctionType::variadic(Some($ret))
    };

    // 2. Variadic without return type: function_type!(*)
    ((*)) => {
        $crate::FunctionType::variadic(None)
    };

    // 3. Named parameters with return type: function_type!((a: TypeA, b: TypeB) -> ReturnTy)
    (($( $param:ident : $ty:expr ),*) -> $ret:expr) => {
        $crate::FunctionType::new(
            $crate::CallSignature::new(vec![
                $(
                    ($crate::Identifier::from(stringify!($param)), $ty)
                ),*
            ]),
            Some($ret),
        )
    };

    // 4. Named parameters without return type: function_type!((a: TypeA, b: TypeB))
    (($( $param:ident : $ty:expr ),* $(,)?)) => {
        $crate::FunctionType::new(
            $crate::CallSignature::new(vec![
                $(
                    ($crate::Identifier::from(stringify!($param)), $ty)
                ),*
            ]),
            None,
        )
    };
}
