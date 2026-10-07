// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad parameter syntax elements

use crate::ir;

use microcad_lang_base::{Identifier, SrcRef, boxed};
use microcad_lang_types::Ty;
use microcad_macros::{Identifiable, SrcReferrer};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Default, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ParameterAttributes {
    pub doc: ir::DocBlock,
}

/// A parameter of a parameter list.
#[derive(Debug, Clone, Hash, SrcReferrer, Identifiable, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter attributes
    pub attr: ParameterAttributes,
    /// Name of the parameter
    pub id: Identifier,
    /// Type of the parameter
    pub ty: ir::Type,
    /// default value of the parameter or `None`
    pub default_value: Option<ir::ConstantExpression>,
    /// Source code reference
    pub src_ref: SrcRef,
}

/// Builder methods.
impl Parameter {
    pub fn new(id: impl AsRef<str>, ty: impl Into<ir::Type>) -> Self {
        Self {
            attr: Default::default(),
            id: Identifier::from(id.as_ref()),
            ty: ty.into(),
            default_value: None,
            src_ref: SrcRef::none(),
        }
    }

    pub fn with_default(mut self, expr: impl Into<ir::ConstantExpression>) -> Self {
        self.default_value = Some(expr.into());
        self
    }
}

/// Returns the type for this parameter (assuming the default value has been evaluated).
impl Ty for Parameter {
    fn ty(&self) -> microcad_lang_types::Type {
        use crate::ir::ExprSpec;
        self.default_value
            .as_ref()
            .map(|value| value.value().map(|value| value.ty()).unwrap_or_default())
            .unwrap_or(self.ty.ty.clone())
    }
}

/// Parameter list, sorted by id.
#[derive(Debug, Clone, SrcReferrer, Hash, PartialEq, Serialize, Deserialize)]
pub struct ParameterList {
    pub parameters: Box<[ir::Parameter]>,
    pub src_ref: SrcRef,
}

impl ParameterList {
    /// Positional lookup by index
    pub fn get_by_index(&self, index: usize) -> Option<&Parameter> {
        self.parameters.get(index)
    }

    /// Named lookup by Identifier
    pub fn get_by_name(&self, id: &Identifier) -> Option<&Parameter> {
        self.parameters.iter().find(|p| &p.id == id)
    }

    /// Total parameter count
    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    /// Check if the `ParameterList` is empty.
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    /// Returns an iterator over the parameters.
    pub fn iter(&self) -> std::slice::Iter<'_, ir::Parameter> {
        self.parameters.iter()
    }

    /// Returns a mutable iterator over the parameters.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, ir::Parameter> {
        self.parameters.iter_mut()
    }

    /// Return an iterator over parameters
    pub(crate) fn names(&self) -> impl Iterator<Item = &ir::Identifier> {
        self.iter().map(|param| &param.id)
    }
}

impl<P> FromIterator<P> for ParameterList
where
    P: Into<Parameter>,
{
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self {
            parameters: boxed(iter.into_iter().map(|p| p.into())),
            src_ref: SrcRef::none(),
        }
    }
}

impl From<Vec<ir::Parameter>> for ParameterList {
    fn from(params: Vec<ir::Parameter>) -> Self {
        Self {
            parameters: params.into_boxed_slice(),
            src_ref: SrcRef::none(),
        }
    }
}
