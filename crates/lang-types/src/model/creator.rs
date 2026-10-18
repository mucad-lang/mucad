// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Creator of work pieces.

//use mucad_package::SymbolId;

use crate::Arguments;
use mucad_lang_base::{BuiltinId, HashId, SrcRef, SymbolId, hash_id};
use serde::{Deserialize, Serialize};

/// A creator is the symbol
#[derive(Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct Creator {
    /// Symbol.
    pub symbol: SymbolId,
    /// Workpiece arguments.
    pub arguments: Arguments,
    /// Hash id
    pub hash_id: HashId,
    /// Symbol ref of the creator. For built-ins, this is none
    pub src_ref: SrcRef,
}

impl Creator {
    /// Create new builtin creator.
    pub fn builtin(symbol: BuiltinId, arguments: impl Into<Arguments>) -> Self {
        let symbol: SymbolId = symbol.into();
        let arguments = arguments.into();
        let hash_id = hash_id!(symbol, arguments);

        Self {
            symbol,
            arguments,
            hash_id,
            src_ref: SrcRef::none(),
        }
    }
}

impl std::fmt::Display for Creator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.arguments, self.hash_id)
    }
}
