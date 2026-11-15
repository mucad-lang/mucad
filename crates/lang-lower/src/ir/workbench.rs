// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Workbench definition syntax element

use crate::{
    CastInto,
    ir::{self, DocBlock},
};

use derive_more::{Display, From};
use mucad_lang_base::{
    Identifier, SingleIdentifier, SrcRef, SrcReferrer, VersionAnnotation, boxed,
};

pub use mucad_lang_base::element::WorkbenchKind;
use mucad_lang_types::Value;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

pub type WorkbenchCall = ir::Call<ir::WorkbenchExpression>;
pub type WorkbenchIf = ir::If<ir::WorkbenchExpression>;

/// Expressions of attributes that can be attached to a model.
#[derive(Debug, Default, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct ModelAttributes {
    pub color: Option<ir::ConstantExpression>,
    pub resolution: Option<ir::ConstantExpression>,
    pub name: Option<ir::ConstantExpression>,
    pub layer: Option<ir::ConstantExpression>,
}

/// A statement of a workbench, which eventually evals into a model or a property of it.
// A property is larger by its documentation and version annotation, but not much: the other
// statements are not smaller than before, when all statements had all fields.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Clone, Hash, From, Serialize, Deserialize)]
pub enum WorkbenchStatement {
    /// An expression of a child model, e.g. `Circle(r = 42mm);`.
    Expression(ExpressionStatement),
    /// An input or output property, e.g. `prop a = 42mm;`.
    Prop(PropertyStatement),
    /// An assignment of a hidden property, e.g. `a = 42mm;`.
    Assignment(AssignmentStatement),
}

/// An expression of a child model, e.g. `Circle(r = 42mm);`.
#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub attr: ModelAttributes,
    pub src_ref: SrcRef,
    pub expression: WorkbenchExpression,
}

/// An input or output property, e.g. `prop a = 42mm;`, which is part of the API of the
/// workbench.
#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct PropertyStatement {
    pub attr: ModelAttributes,
    /// The documentation, e.g. `/// The radius.` above `prop r = 42mm;`.
    pub doc: DocBlock,
    /// The version annotation, e.g. of `#[deprecated] prop a = 42mm;`.
    pub ver: VersionAnnotation,
    pub src_ref: SrcRef,
    /// The source reference of the `prop` keyword.
    pub keyword_src_ref: SrcRef,
    pub name: ir::Identifier,
    pub ty: ir::Type,
    pub expression: WorkbenchExpression,
}

/// An assignment of a hidden property, e.g. `a = 42mm;`.
#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct AssignmentStatement {
    pub attr: ModelAttributes,
    pub src_ref: SrcRef,
    pub name: ir::Identifier,
    pub ty: ir::Type,
    pub expression: WorkbenchExpression,
}

/// The parts, which all statements have.
impl WorkbenchStatement {
    /// The attributes of the model or the property, e.g. `#[color = "red"]`.
    pub fn attr(&self) -> &ModelAttributes {
        match self {
            Self::Expression(statement) => &statement.attr,
            Self::Prop(statement) => &statement.attr,
            Self::Assignment(statement) => &statement.attr,
        }
    }

    /// The attributes of the model or the property, e.g. `#[color = "red"]`.
    pub fn attr_mut(&mut self) -> &mut ModelAttributes {
        match self {
            Self::Expression(statement) => &mut statement.attr,
            Self::Prop(statement) => &mut statement.attr,
            Self::Assignment(statement) => &mut statement.attr,
        }
    }

    /// The expression of the model or the value of the property.
    pub fn expression(&self) -> &WorkbenchExpression {
        match self {
            Self::Expression(statement) => &statement.expression,
            Self::Prop(statement) => &statement.expression,
            Self::Assignment(statement) => &statement.expression,
        }
    }

    /// The expression of the model or the value of the property.
    pub fn expression_mut(&mut self) -> &mut WorkbenchExpression {
        match self {
            Self::Expression(statement) => &mut statement.expression,
            Self::Prop(statement) => &mut statement.expression,
            Self::Assignment(statement) => &mut statement.expression,
        }
    }

    /// The name of a property, `None` for an expression.
    pub fn name(&self) -> Option<&ir::Identifier> {
        match self {
            Self::Expression(_) => None,
            Self::Prop(statement) => Some(&statement.name),
            Self::Assignment(statement) => Some(&statement.name),
        }
    }
}

impl SrcReferrer for WorkbenchStatement {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Expression(statement) => statement.src_ref,
            Self::Prop(statement) => statement.src_ref,
            Self::Assignment(statement) => statement.src_ref,
        }
    }
}

/// Builder methods for testing
impl WorkbenchStatement {
    /// Construct an expression that will eventually produce a model, `__mu::geo2d::Circle(radius = 4.0mm)`.
    pub fn expr(expr: impl Into<WorkbenchExpression>) -> Self {
        Self::from(expr.into())
    }

    /// Construct a property, `prop a = 42mm`.
    pub fn prop(name: impl AsRef<str>, expr: impl Into<WorkbenchExpression>) -> Self {
        Self::Prop(PropertyStatement {
            attr: Default::default(),
            doc: Default::default(),
            ver: Default::default(),
            src_ref: Default::default(),
            keyword_src_ref: Default::default(),
            name: Identifier::from(name.as_ref()),
            ty: Default::default(),
            expression: expr.into(),
        })
    }

    /// Construct an assignment, `a = 42mm`.
    pub fn assignment(name: impl AsRef<str>, expr: impl Into<WorkbenchExpression>) -> Self {
        Self::Assignment(AssignmentStatement {
            attr: Default::default(),
            src_ref: Default::default(),
            name: Identifier::from(name.as_ref()),
            ty: Default::default(),
            expression: expr.into(),
        })
    }
}

impl From<WorkbenchExpression> for WorkbenchStatement {
    fn from(expression: WorkbenchExpression) -> Self {
        Self::Expression(ExpressionStatement {
            attr: Default::default(),
            src_ref: Default::default(),
            expression,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Group {
    pub statements: Box<[WorkbenchStatement]>,
    pub attr: Box<ModelAttributes>,
    pub src_ref: SrcRef,
}

impl Group {
    pub fn new(statements: impl IntoIterator<Item = WorkbenchStatement>) -> Self {
        Self {
            statements: boxed(statements),
            attr: Default::default(),
            src_ref: Default::default(),
        }
    }
}

/// Each WorkbenchStatement eventually evals into a [`Models`]
#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct InitStatement {
    /// Property name
    pub name: ir::Identifier,
    /// Property value
    pub expression: WorkbenchExpression,
    /// Source code reference
    pub src_ref: SrcRef,
}

/// Builder methods for testing
impl InitStatement {
    pub fn new(name: impl AsRef<str>, expr: impl Into<WorkbenchExpression>) -> Self {
        Self {
            name: Identifier::from(name.as_ref()),
            expression: expr.into(),
            src_ref: SrcRef::none(),
        }
    }

    pub fn with_src_ref(mut self, src_ref: SrcRef) -> Self {
        self.src_ref = src_ref;
        self
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct InitAttributes {
    pub doc: DocBlock,
    pub ver: VersionAnnotation,
}

/// A workbench initializer that can be called.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Init {
    /// Parameter list for this init definition
    pub parameters: ir::ParameterList,
    /// Body if the init definition
    pub statements: Box<[InitStatement]>,
    /// SrcRef of the `init` keyword
    pub keyword_ref: SrcRef,
    /// Attributes.
    pub attr: InitAttributes,
    /// Source reference
    pub src_ref: SrcRef,
}

impl Init {
    /// Returns if this is the initializer of the building plan, which lowering adds without
    /// `init` keyword, see [`Init::default_init`].
    pub fn is_implicit(&self) -> bool {
        self.keyword_ref.is_none()
    }

    /// Find init statement by its name.
    pub fn find_statement(&self, name: impl AsRef<str>) -> Option<&InitStatement> {
        self.statements
            .iter()
            .find(|stmt| &stmt.name == name.as_ref())
    }
}

/// Builder methods for testing
impl Init {
    /// Construct new initializer
    pub fn new(parameters: impl Into<ir::ParameterList>) -> Self {
        Self {
            parameters: parameters.into(),
            statements: Default::default(),
            keyword_ref: Default::default(),
            attr: Default::default(),
            src_ref: Default::default(),
        }
    }

    /// Construct new default initializer
    pub fn default_init(parameters: impl Into<ir::ParameterList>) -> Self {
        let parameters = parameters.into();
        let statements: Vec<_> = parameters
            .iter()
            .map(|parameter| {
                ir::InitStatement::new(
                    parameter.id.clone(),
                    ir::Path::Resolved(mucad_lang_base::SymbolId::Local(parameter.id.id().clone())),
                )
            })
            .collect();
        Self::new(parameters).with_statements(statements)
    }

    /// Set statements of this initializer
    pub fn with_statements(mut self, statements: impl IntoIterator<Item = InitStatement>) -> Self {
        self.statements = boxed(statements);
        self
    }
}

/// Node marker, e.g. `@input`.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Hash, Deserialize)]
#[display("@{}", id)]
pub struct Marker {
    /// Marker name, e.g. `input`
    pub id: ir::Identifier,
    /// Source code reference
    pub src_ref: SrcRef,
}

impl Marker {
    /// Create a new input marker
    pub fn input() -> Self {
        Self {
            id: "input".into(),
            src_ref: SrcRef::none(),
        }
    }

    /// Returns true if the marker is an input placeholder
    pub fn is_input_placeholder(&self) -> bool {
        &self.id == "input"
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, From, PartialEq, IntoStaticStr, Hash, Serialize, Deserialize)]
pub enum WorkbenchExpression {
    Error(SrcRef),
    Value(ir::ConstantValue),
    Path(ir::Path),
    Group(ir::Group),
    If(ir::If<WorkbenchExpression>),
    Call(ir::Call<WorkbenchExpression>),
    Marker(Marker),
}

impl WorkbenchExpression {
    pub(crate) fn local(id: &str) -> Self {
        Self::Path(ir::Path::Resolved(mucad_lang_base::SymbolId::Local(
            id.into(),
        )))
    }
}

impl From<Value> for WorkbenchExpression {
    fn from(value: Value) -> Self {
        ir::ConstantValue::new(value).into()
    }
}

impl SrcReferrer for WorkbenchExpression {
    fn src_ref(&self) -> SrcRef {
        use WorkbenchExpression::*;
        match &self {
            Error(src_ref) => *src_ref,
            Value(literal) => literal.src_ref(),
            Path(name) => name.src_ref(),
            Group(group) => group.src_ref,
            If(if_) => if_.src_ref,
            Call(call) => call.src_ref,
            Marker(marker) => marker.src_ref,
        }
    }
}

impl ir::ExprSpec for WorkbenchExpression {
    type Body = Group;

    fn value(&self) -> Option<&Value> {
        match &self {
            WorkbenchExpression::Value(constant) => Some(constant.value()),
            _ => None,
        }
    }
}

impl SingleIdentifier for WorkbenchExpression {
    fn single_identifier(&self) -> Option<&mucad_lang_base::Identifier> {
        match self {
            WorkbenchExpression::Path(path) => path.single_identifier(),
            _ => None,
        }
    }
}

impl CastInto<ir::WorkbenchExpression> for ir::ConstantExpression {
    fn cast_into(self: ir::ConstantExpression) -> ir::WorkbenchExpression {
        match self {
            ir::ConstantExpression::Error(src_ref) => ir::WorkbenchExpression::Error(src_ref),
            ir::ConstantExpression::Value(literal) => ir::WorkbenchExpression::Value(literal),
            ir::ConstantExpression::Path(name) => ir::WorkbenchExpression::Path(name),
            ir::ConstantExpression::Call(call) => ir::WorkbenchExpression::Call(call.cast_into()),
        }
    }
}

impl From<ir::ConstantExpression> for WorkbenchExpression {
    fn from(value: ir::ConstantExpression) -> Self {
        value.cast_into()
    }
}

/// A workbench signature consists of the workbench kind, a parameter list and statements
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchSignature {
    /// Workbench Kind
    pub kind: ir::WorkbenchKind,
    /// Workbench's building plan.
    pub parameters: ir::ParameterList,
    /// `init`
    pub inits: Box<[ir::Init]>,
}

/// Workbench definition, e.g `sketch`, `part` or `op`.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Workbench {
    /// Workbench Kind
    pub signature: WorkbenchSignature,
    /// The actual statements to build the Model
    pub statements: Box<[ir::WorkbenchStatement]>,
}

/// Builder methods for testing
impl Workbench {
    /// Create a new Workbench signature
    pub fn new(kind: WorkbenchKind, parameters: impl Into<ir::ParameterList>) -> Self {
        let parameters = parameters.into();
        Self {
            signature: WorkbenchSignature {
                kind,
                inits: boxed([Init::default_init(parameters.clone())]), // Default init
                parameters,
            },
            statements: Default::default(),
        }
    }

    pub fn sketch(parameters: impl Into<ir::ParameterList>) -> Self {
        Self::new(WorkbenchKind::Sketch, parameters)
    }

    pub fn part(parameters: impl Into<ir::ParameterList>) -> Self {
        Self::new(WorkbenchKind::Part, parameters)
    }

    pub fn op(parameters: impl Into<ir::ParameterList>) -> Self {
        Self::new(WorkbenchKind::Op, parameters)
    }

    pub fn with_inits(mut self, inits: impl IntoIterator<Item = ir::Init>) -> Self {
        // 1. New inits go at the front (indices 0..N)
        let mut combined: Vec<_> = inits.into_iter().collect();

        // 2. Original self.inits are placed AFTER the new inits
        combined.extend(self.signature.inits.into_vec());

        self.signature.inits = combined.into_boxed_slice();
        self
    }

    /// Set statements of this workbench
    pub fn with_statements(
        mut self,
        statements: impl IntoIterator<Item = WorkbenchStatement>,
    ) -> Self {
        self.statements = boxed(statements);
        self
    }
}
