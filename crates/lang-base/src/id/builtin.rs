// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! lang-base/src/builtin.rs

use derive_more::{Debug, Display};
use mucad_hash::HashId;
use serde::{Deserialize, Serialize};

use crate::VersionAnnotation;

/// Strongly-typed wrapper around raw builtin u64 hashes
#[derive(
    Debug, Copy, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BuiltinId(pub HashId);

impl BuiltinId {
    /// FNV-1a compile-time hashing function
    pub const fn from_name(name: &str) -> Self {
        Self(HashId::compile_time_hash(name))
    }
}

impl<'a> From<&'a str> for BuiltinId {
    fn from(name: &'a str) -> Self {
        Self::from_name(name)
    }
}

// Allow conversion directly from String
impl From<String> for BuiltinId {
    #[inline]
    fn from(name: String) -> Self {
        Self::from_name(name.as_str())
    }
}

/// Information about a built-in symbol.
///
/// Each built-in symbol has a [`BuiltinInfo`] as property.
#[derive(Debug, Clone, Display)]
#[debug("{name}")]
#[display("{name}@{id}")]
pub struct BuiltinInfo {
    /// Full name of the built-in, e.g. `__mu::geo2d::Circle`.
    pub name: &'static str,
    /// The hash ID of the built-in, generated from it's full-name.
    pub id: BuiltinId,
    /// Optional documentation.
    pub doc: Option<&'static str>,
    /// Function to get version annotation
    pub version_annotation: Option<fn() -> VersionAnnotation>,
}

/// Builder methods.
impl BuiltinInfo {
    pub const fn new(name: &'static str) -> Self {
        Self {
            id: BuiltinId::from_name(name),
            name,
            doc: None,
            version_annotation: None,
        }
    }

    pub const fn with_doc(mut self, doc: &'static str) -> Self {
        if !doc.is_empty() {
            self.doc = Some(doc);
        }
        self
    }

    pub const fn with_version_annotation(
        mut self,
        version_annotation: fn() -> VersionAnnotation,
    ) -> Self {
        self.version_annotation = Some(version_annotation);
        self
    }
}

/// Getter methods.
impl BuiltinInfo {
    /// Return the Id of the built-in.
    pub const fn id(&self) -> BuiltinId {
        self.id
    }

    /// Returns the module name containing the item (e.g., `"geo2d"` from `"__mu::geo2d::Circle"`).
    /// Returns `""` if there is no parent module.
    pub fn module_name(&self) -> &'static str {
        self.name.rsplit_once("::").map_or("", |(parent, _)| {
            parent
                .rsplit_once("::")
                .map_or(parent, |(_, module)| module)
        })
    }

    /// Returns the item name (e.g., `"Circle"` from `"__mu::geo2d::Circle"`).
    /// Returns `None` if the name only contains a root/module without a distinct item.
    pub fn item_name(&self) -> &'static str {
        self.name
            .rsplit_once("::")
            .map_or(self.name, |(_, item)| item)
    }

    /// Get the first line of the documentation, if any.
    pub fn doc_summary(&self) -> &'static str {
        self.doc
            .and_then(|doc| doc.lines().next())
            .map(str::trim)
            .unwrap_or("")
    }
}

#[macro_export]
macro_rules! builtin_info {
    // Helper: Modules (Only takes a module identifier)
    (
        $doc:literal
        $mod_name:ident
    ) => {
        $crate::BuiltinInfo::new(concat!("__mu::", stringify!($mod_name))).with_doc($doc)
    };

    // Helper: Functions/Constants
    (
        $doc:literal
        $mod_name:ident::$fn_name:ident
    ) => {
        $crate::BuiltinInfo::new(concat!(
            "__mu::",
            stringify!($mod_name),
            "::",
            stringify!($fn_name)
        ))
        .with_doc($doc)
    };
}
