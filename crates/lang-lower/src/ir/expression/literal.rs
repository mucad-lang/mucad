// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad literal syntax elements

use derive_more::Display;
use microcad_lang_base::SrcRef;
use microcad_lang_types::{Type, Value, ty};
use microcad_macros::SrcReferrer;
use serde::{Deserialize, Serialize};

/// A constant value lowered from a ast::Literal.
#[derive(Clone, SrcReferrer, Display, Debug, Hash, PartialEq, Serialize, Deserialize)]
#[display("{}", self.value())]
pub struct ConstantValue {
    pub value: Value,
    pub src_ref: SrcRef,
}

impl ConstantValue {
    pub fn new(value: impl Into<Value>) -> Self {
        Self {
            value: value.into(),
            src_ref: Default::default(),
        }
    }

    pub fn with_src_ref(mut self, src_ref: SrcRef) -> Self {
        self.src_ref = src_ref;
        self
    }

    /// Return value of literal.
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl From<Value> for ConstantValue {
    fn from(v: Value) -> Self {
        Self::new(v)
    }
}

impl ty::Ty for ConstantValue {
    fn ty(&self) -> Type {
        self.value().ty()
    }
}
