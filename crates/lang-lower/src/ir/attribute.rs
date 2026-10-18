// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Attribute syntax entities.

use crate::ir;

use mucad_lang_base::{SingleIdentifier, SrcRef, SrcReferrer};

use mucad_macros::SrcReferrer;
use serde::{Deserialize, Serialize};

/// Block of documentation comments, stripped of `/// `.
#[derive(Clone, Debug, Default, Hash, SrcReferrer, PartialEq, Serialize, Deserialize)]
pub struct DocBlock {
    pub content: String,
    pub src_ref: SrcRef,
}

impl DocBlock {
    /// Check if this doc block is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Merge two doc blocks, e.g. for merging inner and outer docs
    pub fn merge(a: &DocBlock, b: &DocBlock) -> DocBlock {
        match (a.is_empty(), b.is_empty()) {
            (true, true) => Self::default(),
            (true, false) => b.clone(),
            (false, true) => a.clone(),
            _ => Self {
                content: format!("{}\n{}", a.content, b.content),
                src_ref: SrcRef::merge(&a.src_ref, &b.src_ref),
            },
        }
    }
}

/// Key-value attribute pair, e.g. `#[layer = "Test"]`
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct KvExpr {
    pub name: ir::Path,
    pub expr: ir::ConstantExpression,
    pub src_ref: SrcRef,
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub path: ir::Path,
    pub argument_list: ir::ArgumentList<ir::ConstantExpression>,
    pub src_ref: SrcRef,
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: ir::Identifier,
}

impl SrcReferrer for Tag {
    fn src_ref(&self) -> SrcRef {
        self.name.src_ref()
    }
}

/// Crate internal struct for attributes.
///
/// These attributes will be desugared into item-specific attributes.
#[derive(Debug, Default, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) struct Attributes {
    /// Documentation
    pub doc: ir::DocBlock,
    /// Key-value expressions: #[color = "red"]
    pub kv_exprs: Box<[KvExpr]>,
    /// Commands: #[export("file.svg")] #[deprecated(since = "0.2.0")]
    pub commands: Box<[Command]>,
    /// Tags: #[deprecated]
    pub tags: Box<[Tag]>,
}

impl Attributes {
    /// Consumes both `self` and `rhs`, returning a merged `Attributes` with appended fields.
    pub fn extend(mut self, rhs: Self) -> Self {
        /// Helper function to combine two `Box<[T]>` without reallocating if one side is empty.
        fn extend_boxed_slices<T>(lhs: Box<[T]>, rhs: Box<[T]>) -> Box<[T]> {
            if rhs.is_empty() {
                return lhs;
            }
            if lhs.is_empty() {
                return rhs;
            }

            let mut vec = lhs.into_vec();
            vec.extend(rhs.into_vec());
            vec.into_boxed_slice()
        }

        // Extend documentation
        self.doc = ir::DocBlock::merge(&self.doc, &rhs.doc);

        // Convert Box<[T]> to Vec<T> to append, then back to Box<[T]>
        self.kv_exprs = extend_boxed_slices(self.kv_exprs, rhs.kv_exprs);
        self.commands = extend_boxed_slices(self.commands, rhs.commands);
        self.tags = extend_boxed_slices(self.tags, rhs.tags);

        self
    }

    pub fn fetch_kv_expr(&self, arg: impl AsRef<str>) -> Option<ir::ConstantExpression> {
        let arg = arg.as_ref();
        self.kv_exprs
            .iter()
            .find_map(|KvExpr { name, expr, .. }| match name.single_identifier() {
                Some(ident) => {
                    if ident.as_str() == arg {
                        Some(expr.clone())
                    } else {
                        None
                    }
                }
                None => None,
            })
    }
}
