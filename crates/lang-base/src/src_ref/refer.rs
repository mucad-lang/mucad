// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::src_ref::*;
use derive_more::{Deref, DerefMut};

use serde::Serialize;

/// Packs any value together with a source reference
#[derive(
    Clone, Debug, Default, Ord, PartialEq, PartialOrd, Deref, DerefMut, Serialize, Deserialize,
)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct Refer<T> {
    /// Value
    #[deref]
    #[deref_mut]
    pub value: T,
    /// Source code reference
    pub src_ref: SrcRef,
}

impl<T> Refer<T> {
    /// Create a `Refer` instance without source code reference
    pub fn none(value: T) -> Self {
        Self {
            value,
            src_ref: SrcRef::none(),
        }
    }
    /// Create a `Refer` instance with source code reference
    pub fn new(value: T, src_ref: SrcRef) -> Self {
        Self { value, src_ref }
    }
}

impl<T> SrcReferrer for Refer<T> {
    fn src_ref(&self) -> SrcRef {
        self.src_ref
    }
}

impl<T: Eq> Eq for Refer<T> {}

impl<T: std::fmt::Display> std::fmt::Display for Refer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Refer<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: std::iter::IntoIterator> std::iter::IntoIterator for Refer<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<T> From<Refer<T>> for SourceSpan {
    fn from(value: Refer<T>) -> Self {
        value
            .src_ref
            .as_miette_span()
            .unwrap_or_else(|| SourceSpan::new(0.into(), 0))
    }
}
