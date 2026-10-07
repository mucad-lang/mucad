// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language types and values.

pub mod math;
pub mod model;
pub mod ty;
pub mod value;

mod color;
pub use color::Color;

mod arguments;
pub use arguments::{ArgumentValue, ArgumentValueList, Arguments};

use derive_more::Display;
use serde::{Deserialize, Serialize};
pub use ty::{
    CallParameter, CallParameters, CallSignature, FunctionType, MatrixType, QuantityType,
    TupleType, Ty, Type, TypeError, TypeResult, Unit,
};
pub use value::{
    List, Mat2, Mat3, Mat4, Matrix, Quantity, Tuple, Value, ValueAccess, ValueError, ValueList,
    ValueResult,
};

pub use model::{Model, ModelTree, ModelType, NodeMut as ModelNodeMut, NodeRef as ModelNodeRef};

pub use math::MathOps;

pub use microcad_lang_base::{
    Identifier,
    element::{BinaryOperator, UnaryOperator},
};

use crate::math::ScalarF;

pub type Integer = fixed::FixedI64<fixed::types::extra::U0>;

pub type Scalar = fixed::FixedI128<fixed::types::extra::U32>;

/// 2D vector type.
pub type Vec2 = cgmath::Vector2<Scalar>;
/// 3D vector type.
pub type Vec3 = cgmath::Vector3<Scalar>;
/// 4D vector type.
pub type Vec4 = cgmath::Vector4<Scalar>;

/// Primitive angle type in radians.
pub type Angle = cgmath::Rad<Scalar>;

/// A length in mm
#[derive(
    Clone,
    Debug,
    Display,
    Copy,
    Default,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
#[display("{_0}mm")]
pub struct Length(pub Scalar);

impl Length {
    /// Return a new length from millimeters.
    pub fn mm(mm: ScalarF) -> Self {
        Self(Scalar::from_num(mm))
    }

    /// Return the length in millimeters as an f64.
    pub fn as_mm(&self) -> ScalarF {
        self.0.to_num()
    }
}

/// A trait to implement binary and operators for [`Type`] and [`Value`].
pub trait Operators: Sized {
    type Err;

    /// Perform a binary operation.
    fn binary_op(self, op: BinaryOperator, rhs: Self) -> Result<Self, Self::Err>;

    /// Perform a unary operation.
    fn unary_op(self, op: UnaryOperator) -> Result<Self, Self::Err>;
}
