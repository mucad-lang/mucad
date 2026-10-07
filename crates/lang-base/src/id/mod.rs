// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module to handle IDs to reference items in the µcad language.

mod builtin;
mod external;
mod library;

use derive_more::{Display, From};
use indextree::NodeId;
pub use microcad_hash::{HashId, hash_id};

pub use builtin::{BuiltinId, BuiltinInfo};
pub use external::ExternalId;
pub use library::{LibraryId, LibraryInfo};

use serde::{Deserialize, Serialize};

use crate::display::shorten;

/// Name type (base of all identifiers)
pub type Name = crate::CompactString;

/// Symbol id
#[derive(Debug, Display, Hash, PartialEq, Eq, From, Clone, Serialize, Deserialize)]
pub enum SymbolId {
    /// A builtin symbol.
    Builtin(BuiltinId),
    /// A name of a local variable in the current scope.
    Local(Name),
    /// A definition within the current package/module (e.g. a workbench or function).
    Item(NodeId),
    /// A symbol in an external package (e.g. `std`)
    External(ExternalId),
}

/// Trait to look up a human-readable name for a `SymbolId`.
pub trait LookUpName {
    fn look_up_built_in_name(&self, _builtin_id: &BuiltinId) -> Option<Name> {
        None
    }

    fn look_up_item_name(&self, _item_id: &NodeId) -> Option<Name> {
        None
    }

    fn look_up_external_item_name(&self, _exteral_id: &ExternalId) -> Option<Name> {
        None
    }

    fn look_up_symbol_name(&self, symbol_id: &SymbolId) -> Option<Name> {
        match symbol_id {
            SymbolId::Builtin(builtin_id) => self.look_up_built_in_name(builtin_id),
            SymbolId::Local(name) => Some(name.clone()),
            SymbolId::Item(node_id) => self.look_up_item_name(node_id),
            SymbolId::External(external_id) => self.look_up_external_item_name(external_id),
        }
    }

    fn look_up_name(&self) -> Option<Name> {
        None
    }
}

/// Default type for No Look
pub struct DefaultContext;

impl LookUpName for DefaultContext {}

pub trait DisplayWithCtx<Ctx>: Sized {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result;

    /// Renders the object to a String using the provided context.
    fn to_string_with_ctx(&self, ctx: &Ctx) -> String {
        let mut output = String::new();
        let adapter = DisplayWithCtxHelper { value: self, ctx };

        let _ = std::fmt::write(&mut output, format_args!("{adapter}"));
        output
    }

    /// Only show first line of the string.
    /// The line break `\n` is not included.
    fn to_one_line_with_ctx(&self, ctx: &Ctx) -> String {
        let line: String = self
            .to_string_with_ctx(ctx)
            .lines()
            .map(|line| line.to_string())
            .next()
            .unwrap_or_default();
        shorten(&line, 80)
    }
}

pub struct DisplayWithCtxHelper<'a, T, Ctx> {
    pub value: &'a T,
    // UnsafeCell allows us to extract the mutable reference inside `fmt`
    pub ctx: &'a Ctx,
}

impl<'a, T: DisplayWithCtx<Ctx>, Ctx> std::fmt::Display for DisplayWithCtxHelper<'a, T, Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_with_ctx(f, self.ctx)
    }
}
impl<Ctx: LookUpName> DisplayWithCtx<Ctx> for BuiltinId {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        match ctx.look_up_built_in_name(self) {
            Some(name) => write!(f, "{name}[{id}]", id = self),
            None => write!(f, "!{}", self),
        }
    }
}

impl<Ctx: LookUpName> DisplayWithCtx<Ctx> for NodeId {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        match ctx.look_up_item_name(self) {
            Some(name) => write!(f, "{name}[{id}]", id = hash_id!(self)),
            None => write!(f, "!{}", self),
        }
    }
}

impl<Ctx: LookUpName> DisplayWithCtx<Ctx> for SymbolId {
    fn fmt_with_ctx(&self, f: &mut std::fmt::Formatter<'_>, ctx: &Ctx) -> std::fmt::Result {
        match self {
            SymbolId::Builtin(builtin_id) => builtin_id.fmt_with_ctx(f, ctx),
            SymbolId::Local(name) => write!(f, "@Local({name})"),
            SymbolId::Item(node_id) => node_id.fmt_with_ctx(f, ctx),
            SymbolId::External(_) => todo!(),
        }
    }
}
