// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Function definition syntax element

use crate::{CastInto, ir};

use derive_more::From;
use mucad_lang_base::{SingleIdentifier, SrcRef, SrcReferrer, boxed};
use mucad_lang_types::{CallSignature, FunctionType, Ty, Type, Value};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

pub type FunctionCall = ir::Call<ir::FunctionExpression>;
pub type FunctionIf = ir::If<ir::FunctionExpression>;
pub type FunctionLocalAssignment = ir::LocalAssignment<ir::FunctionExpression>;

/// Parameters and return type of a function
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function's parameters
    pub parameters: ir::ParameterList,
    /// Function's return type
    pub return_type: Option<ir::Type>,
    /// Source code reference
    pub src_ref: SrcRef,
}

impl FunctionSignature {
    /// The function type of the signature: its parameter types and return type,
    /// e.g. `(x: Length) -> Length` for `fn f(x: Length) -> Length`.
    ///
    /// A parameter without type has the type of its constant default value, see [`ir::Parameter`].
    pub fn ty(&self) -> FunctionType {
        let parameters: Vec<_> = self
            .parameters
            .parameters
            .iter()
            .map(|param| (param.id.clone(), param.ty()))
            .collect();
        FunctionType::new(CallSignature::new(parameters), self.return_type())
    }

    /// The type of the result of calling the function, e.g. `Length` for `-> Length`.
    ///
    /// Without declared return type, the type of the result is known when calling: `Any`.
    pub fn return_type(&self) -> Option<Type> {
        Some(
            self.return_type
                .as_ref()
                .map_or(Type::Any, |return_type| return_type.ty.clone()),
        )
    }
}

impl Function {
    /// The type of the result of calling the function, see [`FunctionSignature::return_type`].
    pub fn return_type(&self) -> Option<Type> {
        self.signature.return_type()
    }
}

/// The function type of the signature, see [`FunctionSignature::ty`].
impl Ty for Function {
    fn ty(&self) -> Type {
        Type::Function(self.signature.ty())
    }
}

/// Builder methods for testing.
impl Function {
    pub fn new(parameters: impl Into<ir::ParameterList>) -> Self {
        Self {
            signature: FunctionSignature {
                parameters: parameters.into(),
                return_type: None,
                src_ref: SrcRef::none(),
            },
            statements: Default::default(),
        }
    }

    pub fn with_return_type(mut self, ty: impl Into<ir::Type>) -> Self {
        self.signature.return_type = Some(ty.into());
        self
    }

    pub fn with_statements(
        mut self,
        statements: impl IntoIterator<Item = FunctionStatement>,
    ) -> Self {
        self.statements = boxed(statements);
        self
    }
}

/// A function scope `{}`
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Scope {
    pub statements: Box<[FunctionStatement]>,
    pub src_ref: SrcRef,
}

#[non_exhaustive]
#[derive(Debug, Clone, Hash, From, IntoStaticStr, PartialEq, Serialize, Deserialize)]
pub enum FunctionExpression {
    /// Invalid expression (an error occurred)
    Invalid,
    /// A constant value
    Value(ir::ConstantValue),
    /// A path to a constant
    Path(ir::Path),
    /// A `{}`
    Scope(Scope),
    /// An `if` expression
    If(FunctionIf),
    /// A call to a function
    Call(FunctionCall),
}

impl SingleIdentifier for FunctionExpression {
    fn single_identifier(&self) -> Option<&mucad_lang_base::Identifier> {
        match self {
            FunctionExpression::Path(name) => name.single_identifier(),
            _ => None,
        }
    }
}

impl From<Value> for FunctionExpression {
    fn from(value: Value) -> Self {
        ir::ConstantValue::from(value).into()
    }
}

impl ir::ExprSpec for FunctionExpression {
    type Body = Scope;

    fn value(&self) -> Option<&Value> {
        match &self {
            FunctionExpression::Value(constant) => Some(constant.value()),
            _ => None,
        }
    }
}

impl CastInto<ir::FunctionExpression> for ir::ConstantExpression {
    fn cast_into(self: ir::ConstantExpression) -> ir::FunctionExpression {
        match self {
            ir::ConstantExpression::Invalid => ir::FunctionExpression::Invalid,
            ir::ConstantExpression::Value(literal) => ir::FunctionExpression::Value(literal),
            ir::ConstantExpression::Path(path) => ir::FunctionExpression::Path(path),
            ir::ConstantExpression::Call(call) => ir::FunctionExpression::Call(call.cast_into()),
        }
    }
}

impl SrcReferrer for FunctionExpression {
    fn src_ref(&self) -> SrcRef {
        match &self {
            FunctionExpression::Invalid => SrcRef::none(),
            FunctionExpression::Value(literal) => literal.src_ref(),
            FunctionExpression::Path(name) => name.src_ref(),
            FunctionExpression::Scope(scope) => scope.src_ref,
            FunctionExpression::If(if_expr) => if_expr.src_ref,
            FunctionExpression::Call(call) => call.src_ref,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub expr: Option<FunctionExpression>,
    pub keyword_src_ref: SrcRef,
    pub src_ref: SrcRef,
}

#[non_exhaustive]
#[derive(Debug, Clone, From, Hash, PartialEq, Serialize, Deserialize)]
pub enum FunctionStatement {
    /// `a = 42`
    Local(FunctionLocalAssignment),
    /// `{ a = 23; a }`
    Scope(ir::Scope),
    /// `print("Test");`
    Call(ir::Call<FunctionExpression>),
    /// `if { a } else { b }`
    If(ir::If<FunctionExpression>),
    /// Tail expression: `42`
    Tail(FunctionExpression),
    /// A return statement: `return 42;`
    Return(ReturnStatement),
}

impl SrcReferrer for FunctionStatement {
    fn src_ref(&self) -> SrcRef {
        use FunctionStatement::*;
        match &self {
            Local(local_assignment) => local_assignment.src_ref,
            Call(call) => call.src_ref,
            Scope(scope) => scope.src_ref,
            If(if_) => if_.src_ref,
            Tail(expr) => expr.src_ref(),
            Return(return_statement) => return_statement.src_ref,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Function {
    /// Function signature.
    pub signature: ir::FunctionSignature,
    /// Function statements
    pub statements: Box<[ir::FunctionStatement]>,
}
