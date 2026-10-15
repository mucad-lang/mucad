// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Element of a [`Model`].

use derive_more::{Display, From};
use microcad_lang_base::{BuiltinId, DisplayWithCtx, LookUpName, SrcRef, element::WorkbenchKind};
use serde::{Deserialize, Serialize};

use crate::{Ty, Type, Value, math::AffineTransform, model::ModelType};

/// Boolean operations
#[derive(Clone, Copy, Debug, Display, Hash, PartialEq, Serialize, Deserialize)]
pub enum BooleanOp {
    /// Computes the union R = P ∪ Q
    Union,
    /// computes the difference R = P ∖ Q
    Difference,
    /// computes the intersection R = P ∩ Q
    Intersect,
}

impl BooleanOp {
    fn builtin_id(&self) -> BuiltinId {
        // TODO This is a hack. Check if the BooleanOp variant can be removed eventually.
        match self {
            BooleanOp::Union => "__mu::ops::union",
            BooleanOp::Difference => "__mu::ops::subtract",
            BooleanOp::Intersect => "__mu::ops::intersect",
        }
        .into()
    }
}

#[derive(Debug, Display, Clone, Hash, PartialEq, Serialize, Deserialize)]
#[display("{kind}")]
pub struct Workpiece {
    pub kind: WorkbenchKind,
    // TODO pub symbol_node_id: SymbolNodeId,
    pub src_ref: SrcRef,
}

impl Workpiece {
    pub fn new(kind: WorkbenchKind) -> Self {
        Self {
            kind,
            src_ref: SrcRef::none(),
        }
    }
}

impl Ty for WorkbenchKind {
    fn ty(&self) -> crate::Type {
        match self {
            WorkbenchKind::Sketch => Type::Model(ModelType::Geometry2D),
            WorkbenchKind::Part => Type::Model(ModelType::Geometry3D),
            WorkbenchKind::Op => Type::Model(ModelType::Any),
        }
    }
}

/// The kind of the built-in workbench determines its output.
#[non_exhaustive]
#[derive(Debug, Display, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum BuiltinWorkpiece {
    /// A parametric primitive with its output type (2D or 3D).
    #[display("{_0}")]
    Primitive(BuiltinId, ModelType),
    /// An affine transformation.
    AffineTransform(Box<AffineTransform>),
    /// Boolean operation
    BooleanOp(BooleanOp),
    /// A built-in operation with its output type, e.g. `extrude`.
    ///
    /// The output type is `NotDetermined` if it depends on the input, e.g. for `hull`.
    #[display("{_0}")]
    Operation(BuiltinId, ModelType),
}

impl BuiltinWorkpiece {
    fn output_type(&self) -> ModelType {
        match self {
            BuiltinWorkpiece::Primitive(_, output_type)
            | BuiltinWorkpiece::Operation(_, output_type) => *output_type,
            BuiltinWorkpiece::AffineTransform(_) | BuiltinWorkpiece::BooleanOp(_) => {
                ModelType::NotDetermined
            }
        }
    }

    pub fn builtin_id(&self) -> Option<BuiltinId> {
        match self {
            BuiltinWorkpiece::Primitive(builtin_id, _)
            | BuiltinWorkpiece::Operation(builtin_id, _) => Some(*builtin_id),
            BuiltinWorkpiece::BooleanOp(op) => Some(op.builtin_id()),
            _ => None,
        }
    }
}

impl<Ctx> DisplayWithCtx<Ctx> for BuiltinWorkpiece
where
    Ctx: LookUpName,
{
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        match self {
            BuiltinWorkpiece::Primitive(builtin_id, _) => builtin_id.fmt_with_ctx(f, ctx),
            BuiltinWorkpiece::AffineTransform(affine_transform) => write!(f, "{affine_transform}"),
            BuiltinWorkpiece::BooleanOp(boolean_op) => write!(f, "{boolean_op}"),
            BuiltinWorkpiece::Operation(builtin_id, _) => builtin_id.fmt_with_ctx(f, ctx),
        }
    }
}

/// An element defines the entity of a [`Model`].
#[derive(Clone, Debug, Hash, PartialEq, Default, From, Serialize, Deserialize)]
pub enum Element {
    #[default]
    /// A group element is created by a body `{}`.
    Group,

    /// An element containing a value.
    Value(Value),

    /// A workpiece which is created by workbenches.
    Workpiece(Workpiece),

    /// A built-in workpiece which created by built-in workbenches.
    BuiltinWorkpiece(BuiltinWorkpiece),

    /// Multiplicity.
    Multiplicity,

    /// Created from `@input` marker. Will never be part of the final model.
    InputPlaceholder,
}

impl Element {
    pub fn output_type(&self) -> ModelType {
        use Element::*;
        match &self {
            Workpiece(workpiece) => workpiece.kind.into(),
            BuiltinWorkpiece(builtin_workpiece) => builtin_workpiece.output_type(),
            Group | Multiplicity | InputPlaceholder | Value(_) => ModelType::NotDetermined,
        }
    }
}

impl<Ctx: LookUpName> DisplayWithCtx<Ctx> for Element {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        match &self {
            Element::Group => write!(f, "Group"),
            Element::Value(value) => write!(f, "Value({value})"),
            Element::Workpiece(workpiece) => write!(f, "Workpiece({workpiece})"),
            Element::BuiltinWorkpiece(builtin_workpiece) => {
                write!(f, "Builtin({})", builtin_workpiece.to_string_with_ctx(ctx))
            }
            Element::Multiplicity => write!(f, "Multiplicity"),
            Element::InputPlaceholder => write!(f, "InputPlaceholder"),
        }
    }
}
