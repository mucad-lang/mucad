// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hash functionality.

use derive_more::Deref;

use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use std::fmt;

/// Hash type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashId(u64);

impl HashId {
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    // Simple FNV-1a hash for generating built-in IDs from &str.
    pub const fn compile_time_hash(s: &str) -> Self {
        let bytes = s.as_bytes();
        let mut hash: u64 = 0xcbf29ce484222325;
        let mut i = 0;
        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            i += 1;
        }

        Self(hash)
    }
}

impl std::str::FromStr for HashId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 1. Trim whitespace to be forgiving of user/config input
        let s = s.trim();

        // 2. Strip "0x" or "0X" if present
        let trimmed = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);

        // 3. Parse as hex. If the prefix wasn't there, we still assume hex
        // because of the human-readable formatting.
        u64::from_str_radix(trimmed, 16).map(Self)
    }
}

impl Serialize for HashId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            // Serialize as "0x..." string
            serializer.serialize_str(&format!("{:#x}", self.0))
        } else {
            // Serialize as raw u64 (binary)
            serializer.serialize_u64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for HashId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            // Deserialize from string
            let s = String::deserialize(deserializer)?;
            let trimmed = s.strip_prefix("0x").unwrap_or(&s);
            u64::from_str_radix(trimmed, 16)
                .map(HashId)
                .map_err(serde::de::Error::custom)
        } else {
            // Deserialize from raw u64
            u64::deserialize(deserializer).map(HashId)
        }
    }
}

impl fmt::Display for HashId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

pub use rustc_hash::FxHashMap as HashMap;
pub use rustc_hash::FxHashSet as HashSet;
pub use rustc_hash::FxHasher as Hasher;

/// Trait to implement for hashable types
pub trait ToHash {
    /// Return computed hash value.
    fn to_hash(&self) -> HashId;
}

impl<T> ToHash for T
where
    T: std::hash::Hash,
{
    fn to_hash(&self) -> HashId {
        use std::hash::Hasher;
        let mut hasher = rustc_hash::FxHasher::default();
        self.hash(&mut hasher);
        HashId(hasher.finish())
    }
}

/// Generic wrapper that contains the hashed value.
#[derive(Deref, Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct Hashed<T: std::hash::Hash> {
    #[deref]
    inner: T,
    hash: HashId,
}

impl<T: std::hash::Hash> Hashed<T> {
    /// Create a new wrapper with hashed.
    pub fn new(inner: T) -> Self {
        Self {
            hash: inner.to_hash(),
            inner,
        }
    }

    /// Transforms the inner value and recalculates the hash for the new value.
    pub fn map<U: std::hash::Hash, F>(self, f: F) -> Hashed<U>
    where
        F: FnOnce(T) -> U,
    {
        // Transform the value
        let new_inner = f(self.inner);

        // Re-hash the new value to ensure the HashId stays in sync
        Hashed::new(new_inner)
    }

    /// Return inner value.
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    /// Return inner value.
    pub fn inner(self) -> T {
        self.inner
    }

    pub fn hash(&self) -> HashId {
        self.hash
    }
}

impl Hashed<String> {
    /// Convert a hashed string into a hashed &str.
    pub fn as_str(&self) -> Hashed<&str> {
        Hashed {
            inner: self.inner.as_str(),
            hash: self.hash,
        }
    }
}

impl<T: std::hash::Hash> ToHash for Hashed<T> {
    fn to_hash(&self) -> HashId {
        self.hash
    }
}

/// Macro to calculate hashes conveniently.
#[macro_export]
macro_rules! hash_id {
    ($($arg:expr),+ $(,)?) => {{
        use ::std::hash::{Hash, Hasher};
        let mut hasher = $crate::Hasher::default();
        $(
            $arg.hash(&mut hasher);
        )+
        $crate::HashId::new(hasher.finish())
    }};
}
