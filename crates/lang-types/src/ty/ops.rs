// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{BinaryOperator, Operators, QuantityType, Type, TypeError, TypeResult, UnaryOperator};

impl Type {
    /// <, >, >=, <=
    fn compare(self, op: BinaryOperator, rhs: Self) -> TypeResult {
        use Type::*;
        let lhs = self;

        match (lhs, rhs) {
            (Integer, Integer) => Ok(Bool),
            (Quantity(lhs), Quantity(rhs)) if lhs == rhs => Ok(Bool),
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator { op, lhs, rhs }),
        }
    }

    /// ==, !=
    fn eq(self, op: BinaryOperator, rhs: Self) -> TypeResult {
        use Type::*;
        let lhs = self;

        match (lhs, rhs) {
            (Integer, Integer) => Ok(Bool),
            (Quantity(lhs), Quantity(rhs)) if lhs == rhs => Ok(Bool),
            (String, String) => Ok(Bool),
            (Tuple(lhs), Tuple(rhs)) if lhs == rhs => Ok(Bool),
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator { op, lhs, rhs }),
        }
    }
}

impl Operators for Type {
    type Err = TypeError;

    fn binary_op(self, op: BinaryOperator, rhs: Self) -> TypeResult {
        use BinaryOperator::*;
        let lhs = self;
        match op {
            Add => lhs + rhs,
            Subtract => lhs - rhs,
            Multiply => lhs * rhs,
            Divide => lhs / rhs,
            Union | Or => lhs | rhs,
            Intersect | And => lhs & rhs,
            GreaterThan | LessThan | LessEqual | GreaterEqual => lhs.compare(op, rhs),
            Equal | NotEqual => lhs.eq(op, rhs),
            Near | PowerXor | Xor => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Near,
                lhs,
                rhs,
            }),
        }
    }

    fn unary_op(self, op: UnaryOperator) -> TypeResult {
        match op {
            UnaryOperator::Minus => -self,
            UnaryOperator::Not => !self,
            UnaryOperator::Plus => Ok(self),
        }
    }
}

impl std::ops::Neg for Type {
    type Output = TypeResult;

    fn neg(self) -> Self::Output {
        use Type::*;
        match self {
            Integer | Quantity(_) => Ok(self),
            List(ty) => -(*ty),
            Tuple(ty) => -(*ty),
            ty => Err(TypeError::UnsupportedUnaryOperator {
                op: UnaryOperator::Minus,
                ty,
            }),
        }
    }
}

impl std::ops::Not for Type {
    type Output = TypeResult;

    fn not(self) -> Self::Output {
        use Type::*;
        match self {
            Bool => Ok(self),
            List(ty) => -(*ty),
            Tuple(ty) => -(*ty),
            ty => Err(TypeError::UnsupportedUnaryOperator {
                op: UnaryOperator::Not,
                ty,
            }),
        }
    }
}

impl std::ops::Add for Type {
    type Output = TypeResult;

    fn add(self, rhs: Self) -> Self::Output {
        use Type::*;
        let lhs = self;
        match (lhs, rhs) {
            (Integer, Integer) => Ok(Integer),
            (Integer, Quantity(QuantityType::Scalar)) => Ok(Quantity(QuantityType::Scalar)),
            (Quantity(QuantityType::Scalar), Integer) => Ok(Quantity(QuantityType::Scalar)),
            (Quantity(lhs), Quantity(rhs)) => lhs + rhs,
            (String, String) => Ok(Type::String),
            (List(lhs), List(rhs)) => *lhs + *rhs,
            (Tuple(lhs), Tuple(rhs)) => *lhs + *rhs,
            (Matrix(lhs), Matrix(rhs)) => lhs + rhs,
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Add,
                lhs,
                rhs,
            }),
        }
    }
}

impl std::ops::Sub for Type {
    type Output = TypeResult;

    fn sub(self, rhs: Self) -> Self::Output {
        use Type::*;
        let lhs = self;
        match (lhs, rhs) {
            (Integer, Integer) => Ok(Integer),
            (Integer, Quantity(QuantityType::Scalar)) => Ok(Quantity(QuantityType::Scalar)),
            (Quantity(QuantityType::Scalar), Integer) => Ok(Quantity(QuantityType::Scalar)),
            (Quantity(lhs), Quantity(rhs)) => lhs - rhs,
            (List(lhs), List(rhs)) => *lhs - *rhs,
            (Tuple(lhs), Tuple(rhs)) => *lhs - *rhs,
            (Matrix(lhs), Matrix(rhs)) => lhs - rhs,
            (Model(lhs), Model(rhs)) if lhs == rhs => Ok(Model(lhs)),
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Add,
                lhs,
                rhs,
            }),
        }
    }
}

impl std::ops::Mul for Type {
    type Output = TypeResult;

    fn mul(self, rhs: Self) -> Self::Output {
        use Type::*;
        let lhs = self;
        match (lhs, rhs) {
            (Integer, Integer) => Ok(Integer),
            (Integer, Quantity(QuantityType::Scalar)) => Ok(Quantity(QuantityType::Scalar)),
            (Quantity(QuantityType::Scalar), Integer) => Ok(Quantity(QuantityType::Scalar)),
            (Quantity(lhs), Quantity(rhs)) => lhs * rhs,
            (ty, List(list_type)) | (List(list_type), ty) => *list_type * ty,
            (Tuple(_), _) | (_, Tuple(_)) => todo!(),
            (Matrix(_), _) | (_, Matrix(_)) => todo!(),
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Multiply,
                lhs,
                rhs,
            }),
        }
    }
}

impl std::ops::Div for Type {
    type Output = TypeResult;

    fn div(self, rhs: Self) -> Self::Output {
        use Type::*;
        let lhs = self;

        match (lhs, rhs) {
            (Integer, Integer) => Ok(Integer),
            (Quantity(lhs), Type::Quantity(rhs)) => lhs / rhs,
            (List(list_type), ty) => *list_type / ty,
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Divide,
                lhs,
                rhs,
            }),
        }
    }
}

impl std::ops::BitOr for Type {
    type Output = TypeResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Type::*;
        let lhs = self;
        match (lhs, rhs) {
            (Model(lhs), Model(rhs)) if lhs == rhs => Ok(Model(lhs)),
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Union,
                lhs,
                rhs,
            }),
        }
    }
}

impl std::ops::BitAnd for Type {
    type Output = TypeResult;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Type::*;
        let lhs = self;
        match (lhs, rhs) {
            (Model(lhs), Model(rhs)) if lhs == rhs => Ok(Model(lhs)),
            (lhs, rhs) => Err(TypeError::UnsupportedBinaryOperator {
                op: BinaryOperator::Intersect,
                lhs,
                rhs,
            }),
        }
    }
}
