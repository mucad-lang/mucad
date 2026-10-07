// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad assignment syntax element

use crate::{CastInto, ir};

use microcad_lang_base::{Identifier, SrcRef};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A local assignment specifying an identifier, type and value
#[skip_serializing_none]
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "Expr: Serialize", deserialize = "Expr: Deserialize<'de>"))]
pub struct LocalAssignment<Expr> {
    /// Assignee
    pub id: Identifier,
    /// Type of the assignee
    pub ty: ir::Type,
    /// Value to assign
    pub expression: Expr,
    /// Source code reference
    pub src_ref: SrcRef,
}

impl<T> LocalAssignment<T> {
    /// Maps the inner expression to a new type while preserving node metadata.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> LocalAssignment<U> {
        LocalAssignment {
            id: self.id,
            ty: self.ty,
            expression: f(self.expression),
            src_ref: self.src_ref,
        }
    }
}

macro_rules! impl_cast_into {
    ($ty:ident) => {
        impl<Source, Target> CastInto<$ty<Target>> for $ty<Source>
        where
            Source: $crate::CastInto<Target>,
        {
            fn cast_into(self) -> $ty<Target> {
                self.map(CastInto::cast_into)
            }
        }
    };
}

impl_cast_into!(LocalAssignment);
