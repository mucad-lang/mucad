// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad Type

use derive_more::From;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::{model::ModelType, ty::*};

/// µcad Basic Types
#[derive(
    Clone, Debug, Default, PartialEq, IntoStaticStr, From, Eq, Hash, Serialize, Deserialize,
)]
pub enum Type {
    /// Any type allowed.
    #[default]
    Any,
    /// Invalid type (used for error handling)
    Invalid,
    /// A 64-bit integer number: `Integer: 10`.
    Integer,
    /// A quantity type: `Scalar: 1.0`, `Length: 1.0mm`
    Quantity(QuantityType),
    /// A string.
    String,
    /// A boolean: `true`, `false`.
    Bool,
    /// A list of elements, with a optional: `[Scalar]`.
    /// `List` is a type alias for `[Any]`.
    List(Box<Type>),
    /// A named tuple of elements: `(x: Scalar, y: String)`.
    Tuple(Box<TupleType>),
    /// Matrix type: `Matrix3x3`.
    Matrix(MatrixType),
    /// Function type: (x: Length, y: Length) -> Length,
    Function(FunctionType),
    /// Model.
    Model(ModelType),
}

impl Type {
    /// Shortcut to create a scalar type.
    pub fn scalar() -> Self {
        Self::Quantity(QuantityType::Scalar)
    }

    /// Shortcut to create a length type.
    pub fn length() -> Self {
        Self::Quantity(QuantityType::Length)
    }

    /// Shortcut to create an angle type.
    pub fn angle() -> Self {
        Self::Quantity(QuantityType::Angle)
    }

    /// Short-cut for a `Color` type
    pub fn color() -> Self {
        TupleType::color().into()
    }

    /// Short-cut for a `List` type, or `[Any]`, respectively.
    pub fn list() -> Self {
        Self::List(Box::new(Type::Any))
    }

    pub fn mat3() -> Self {
        Self::Matrix(MatrixType {
            rows: 3,
            columns: 3,
        })
    }

    /// Any model
    pub fn model() -> Self {
        Self::Model(ModelType::Any)
    }

    /// A model containing a 2D geometry
    pub fn model_2d() -> Self {
        Self::Model(ModelType::Geometry2D)
    }

    /// A model containing a 3D geometry
    pub fn model_3d() -> Self {
        Self::Model(ModelType::Geometry3D)
    }

    /// Short-cut for a matrix type.
    pub fn matrix(rows: usize, columns: usize) -> Self {
        Self::Matrix(MatrixType::new(rows, columns))
    }

    /// Check if the type is an list of the given type `ty`
    pub fn is_list_of(&self, ty: &Type) -> bool {
        match self {
            Self::List(list_type) => list_type.as_ref() == ty,
            _ => false,
        }
    }

    /// Check if types are compatible.
    pub fn is_compatible_to(&self, rhs: &Self) -> bool {
        rhs == self
            || (*self == Type::Integer && *rhs == Type::scalar())
            || (*rhs == Type::Integer && *self == Type::scalar())
    }

    /// Returns if the given type or it's inner type matches the given type.
    pub fn matches(&self, other: &Type) -> bool {
        match (self, other) {
            (lhs, Type::Quantity(QuantityType::Scalar)) if *lhs != Type::Any => {
                self == &Type::scalar() || self == &Type::Integer
            }
            // Invalid types never match
            (Type::Tuple(ty_s), Type::Tuple(ty_p)) => ty_s.matches_replication(ty_p),
            // Lists match by their element types, e.g. `[Integer]` matches `[Any]`.
            (Type::List(ty_s), Type::List(ty_p)) => ty_s.matches(ty_p),
            (Type::Invalid, _) | (_, Type::Invalid) => false,
            (Type::Any, _) | (_, Type::Any) => true,
            _ => self == other,
        }
    }

    /// Returns if a list matches the parameter type `param` by its elements (list expansion),
    /// e.g. `[Integer]` matches `Integer`, but `[Integer]` doesn't match `[Integer]` or `Any` this way.
    pub fn matches_list_type(&self, param: &Type) -> bool {
        match self {
            Self::List(ty) => !self.matches(param) && ty.matches(param),
            _ => false,
        }
    }
}

impl std::str::FromStr for Type {
    type Err = TypeError;

    /// Parse a type from &str
    fn from_str(ty: &str) -> Result<Self, TypeError> {
        if let Some(dimensions) = ty.strip_prefix("Matrix") {
            let (x, y) = dimensions
                .split_once('x')
                .unwrap_or((dimensions, dimensions));
            let x = usize::from_str(x).map_err(|_| TypeError::InvalidMatrixType(ty.to_string()))?;
            let y = usize::from_str(y).map_err(|_| TypeError::InvalidMatrixType(ty.to_string()))?;
            return Ok(Type::Matrix(MatrixType::new(x, y)));
        }

        Ok(match ty {
            "Color" => Type::color(),
            "Vec2" => Type::Tuple(Box::new(TupleType::vec2())),
            "Vec3" => Type::Tuple(Box::new(TupleType::vec3())),
            "Mat3" => Type::matrix(3, 3),
            "Size2" => Type::Tuple(Box::new(TupleType::size2())),
            "Any" => Type::Any,
            "Integer" => Type::Integer,
            "Bool" => Type::Bool,
            "String" => Type::String,
            "Scalar" => Type::Quantity(QuantityType::Scalar),
            "Length" => Type::Quantity(QuantityType::Length),
            "Area" => Type::Quantity(QuantityType::Area),
            "Angle" => Type::Quantity(QuantityType::Angle),
            "Volume" => Type::Quantity(QuantityType::Volume),
            "Weight" => Type::Quantity(QuantityType::Weight),
            "Density" => Type::Quantity(QuantityType::Density),
            "Model" => Type::Model(ModelType::Any),
            "Sketch" => Type::Model(ModelType::Geometry2D),
            "Part" => Type::Model(ModelType::Geometry3D),
            _ => return Err(TypeError::UnknownType(ty.to_string())),
        })
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Invalid => write!(f, "(<NO TYPE>)"),
            Self::Quantity(quantity) => write!(f, "{quantity}"),
            Self::List(t) => write!(f, "[{t}]"),
            Self::Tuple(t) => write!(f, "{t}"),
            Self::Matrix(t) => write!(f, "{t}"),
            Self::Function(t) => write!(f, "{t}"),
            Self::Model(output_type) => write!(f, "Model({output_type})"),
            _ => {
                let name: &'static str = self.into();
                write!(f, "{name}")
            }
        }
    }
}

impl From<TupleType> for Type {
    fn from(ty: TupleType) -> Self {
        Self::Tuple(Box::new(ty))
    }
}
