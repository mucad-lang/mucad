// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use derive_more::{Display, From};
use mucad_lang_base::{
    BuiltinId, Identifier, Name, SingleIdentifier, SrcRef, SrcReferrer, SymbolId, ToCompactString,
};
use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A path that needs to be resolved.
#[derive(Clone, Debug, From, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedPath {
    pub is_absolute: bool,
    pub parts: Box<[Identifier]>,
    pub src_ref: SrcRef,
}

impl UnresolvedPath {
    /// Generate a [`BuiltinId`] if the path is not absolute and starts with `__mu`.
    pub fn builtin_id(&self) -> Option<BuiltinId> {
        if let Some(prefix) = self.parts.first()
            && !self.is_absolute
            && prefix.as_str() == "__mu"
        {
            Some(BuiltinId::from(self.to_string().as_str()))
        } else {
            None
        }
    }

    /// Returns the name of an external library, presumably the first part of this path.
    ///
    /// The path must:
    /// * Be relative and not absolute
    /// * If path starts with `mu`, the second path is returned.
    pub fn external_name(&self) -> Option<Name> {
        if !self.is_absolute
            && let Some(part) = self.parts.first()
        {
            if part.as_str() == "mu" {
                self.parts.get(1).map(|part| part.to_compact_string())
            } else {
                Some(part.to_compact_string())
            }
        } else {
            None
        }
    }

    /// The path without a leading `mu`, which marks external libraries:
    /// `mu::std::geo2d::Circle` → `std::geo2d::Circle`.
    pub fn without_mu_prefix(&self) -> UnresolvedPath {
        match self.parts.first() {
            Some(part) if !self.is_absolute && part.as_str() == "mu" => UnresolvedPath {
                is_absolute: false,
                parts: self.parts[1..].into(),
                src_ref: self.src_ref,
            },
            _ => self.clone(),
        }
    }
}

impl SingleIdentifier for UnresolvedPath {
    fn single_identifier(&self) -> Option<&Identifier> {
        if self.is_single_identifier() {
            self.parts.first()
        } else {
            None
        }
    }

    fn is_single_identifier(&self) -> bool {
        !self.is_absolute && self.parts.len() == 1
    }
}

impl std::str::FromStr for UnresolvedPath {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let is_absolute = s.starts_with("::");
        let path_str = if is_absolute { &s[2..] } else { s };

        let parts = path_str
            .split("::")
            .filter(|part| !part.is_empty())
            .map(Identifier::from)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Ok(Self {
            is_absolute,
            parts,
            src_ref: SrcRef::none(),
        })
    }
}
impl std::fmt::Display for UnresolvedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_absolute {
            write!(f, "::")?;
        }
        write!(
            f,
            "{}",
            self.parts
                .iter()
                .map(|part| part.to_string())
                .collect::<Vec<_>>()
                .join("::")
        )
    }
}

#[derive(Clone, Debug, Display, From, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Path {
    /// A Path that has been resolved into a `SymbolId`.
    Resolved(SymbolId),
    /// A path that still needs to be resolved by the symbol resolver.
    Unresolved(UnresolvedPath),
    /// A human readable resolved path.
    #[display("{name}[{id}]")]
    HumanReadable { name: Name, id: SymbolId },
}

impl Path {
    /// Return the symbol id of this path, if it has been resolved
    pub fn symbol_id(&self) -> Option<&SymbolId> {
        match &self {
            Path::Resolved(id) | Path::HumanReadable { id, .. } => Some(id),
            Path::Unresolved(_) => None,
        }
    }
}

impl From<BuiltinId> for Path {
    fn from(id: BuiltinId) -> Self {
        Self::Resolved(SymbolId::Builtin(id))
    }
}

impl SrcReferrer for Path {
    fn src_ref(&self) -> SrcRef {
        match self {
            Path::Unresolved(path) => path.src_ref,
            _ => SrcRef::none(),
        }
    }
}

impl SingleIdentifier for Path {
    fn single_identifier(&self) -> Option<&Identifier> {
        match self {
            Path::Unresolved(path) => path.single_identifier(),
            _ => None,
        }
    }

    fn is_single_identifier(&self) -> bool {
        match self {
            Path::Unresolved(path) => path.is_single_identifier(),
            _ => false,
        }
    }
}

impl From<Path> for SourceSpan {
    fn from(value: Path) -> Self {
        value.src_ref().into()
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        let (is_absolute, s) = if s.starts_with("::") {
            (true, s.strip_prefix("::").unwrap())
        } else {
            (false, s)
        };

        Self::Unresolved(UnresolvedPath {
            is_absolute,
            parts: s.split("::").map(Identifier::from).collect(),
            src_ref: SrcRef::none(),
        })
    }
}

impl From<Identifier> for Path {
    fn from(id: Identifier) -> Self {
        let src_ref = id.src_ref();
        Self::Unresolved(UnresolvedPath {
            is_absolute: false,
            parts: vec![id].into_boxed_slice(),
            src_ref,
        })
    }
}
