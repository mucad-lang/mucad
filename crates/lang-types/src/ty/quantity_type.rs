// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad quantity type

use microcad_lang_base::element::BinaryOperator;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::{TypeError, TypeResult, ty::Unit};

/// A quantity type with
#[derive(
    Clone, Debug, IntoStaticStr, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum QuantityType {
    /// A unitless scalar value.
    Scalar,
    /// Length in mm.
    Length,
    /// Area in mm².
    Area,
    /// Volume in mm³.
    Volume,
    /// Density in g/mm³
    Density,
    /// An angle in radians.
    Angle,
    /// Weight of a specific volume of material.
    Weight,
}

impl QuantityType {
    /// Return base unit
    pub fn base_unit(&self) -> Unit {
        match self {
            QuantityType::Scalar => Unit::None,
            QuantityType::Length => Unit::Millimeter,
            QuantityType::Area => Unit::Millimeter2,
            QuantityType::Volume => Unit::Millimeter3,
            QuantityType::Density => Unit::GramPerMeter3,
            QuantityType::Angle => Unit::Rad,
            QuantityType::Weight => Unit::Gram,
        }
    }
}

impl std::fmt::Display for QuantityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(f, "{name}")
    }
}

impl std::ops::Add for QuantityType {
    type Output = TypeResult;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => lhs,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Add,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        }
        .into())
    }
}

impl std::ops::Sub for QuantityType {
    type Output = TypeResult;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Ok(match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => lhs,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Subtract,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        }
        .into())
    }
}

impl std::ops::Mul for QuantityType {
    type Output = TypeResult;

    fn mul(self, rhs: Self) -> Self::Output {
        use QuantityType::*;
        let lhs = self;
        Ok(match (lhs, rhs) {
            (Length, Length) => Area,
            (ty, Scalar) | (Scalar, ty) => ty,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Multiply,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        }
        .into())
    }
}

impl std::ops::Div for QuantityType {
    type Output = TypeResult;

    fn div(self, rhs: Self) -> Self::Output {
        use QuantityType::*;
        let lhs = self;
        Ok(match (lhs, rhs) {
            (ty, Scalar) => ty,
            (lhs, rhs) if lhs == rhs => Scalar,
            (lhs, rhs) => {
                return Err(TypeError::UnsupportedBinaryOperator {
                    op: BinaryOperator::Divide,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                });
            }
        }
        .into())
    }
}
