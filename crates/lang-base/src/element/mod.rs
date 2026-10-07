// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod identifier;

pub use identifier::{Case, Identifier, IdentifierList, ShortId};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// The possible type of workbenches
#[derive(Debug, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum WorkbenchKind {
    /// `sketch`
    Sketch,
    /// `part`
    Part,
    /// `op`
    Op,
}

impl std::fmt::Display for WorkbenchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                WorkbenchKind::Sketch => "sketch",
                WorkbenchKind::Part => "part",
                WorkbenchKind::Op => "op",
            }
        )
    }
}

/// Visibility of a symbol.
///
/// This is used to determine if an entity is public or private.
/// By default, entities are private.
#[derive(Debug, Hash, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Visibility {
    /// `pub`
    Public,
    /// Everything is private by default.
    #[default]
    Private,
}

impl Visibility {
    /// Returns true if visibility is public.
    pub fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Private => Ok(()),
            Visibility::Public => write!(f, "pub "),
        }
    }
}

/// The type of the operator for binary operations
#[derive(Debug, PartialEq, Hash, Clone, Copy, EnumString, Display, Serialize, Deserialize)]
pub enum BinaryOperator {
    #[strum(serialize = "+")]
    Add,
    #[strum(serialize = "-")]
    Subtract,
    #[strum(serialize = "*")]
    Multiply,
    #[strum(serialize = "/")]
    Divide,
    #[strum(serialize = "|")]
    Union,
    #[strum(serialize = "&")]
    Intersect,
    #[strum(serialize = "^")]
    PowerXor,
    #[strum(serialize = ">")]
    GreaterThan,
    #[strum(serialize = "<")]
    LessThan,
    #[strum(serialize = ">=", serialize = "≥")]
    GreaterEqual,
    #[strum(serialize = "<=", serialize = "≤")]
    LessEqual,
    #[strum(serialize = "==")]
    Equal,
    #[strum(serialize = "~=")]
    Near,
    #[strum(serialize = "!=")]
    NotEqual,
    #[strum(serialize = "and")]
    And,
    #[strum(serialize = "or")]
    Or,
    #[strum(serialize = "xor")]
    Xor,
}

/// The type of the operator for unary operations
#[derive(Debug, PartialEq, Hash, Clone, Copy, Display, EnumString, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum UnaryOperator {
    #[strum(serialize = "-")]
    Minus,
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = "!")]
    Not,
}
