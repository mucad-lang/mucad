// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use compact_str::{CompactString, ToCompactString};
use derive_more::{Deref, DerefMut};
use miette::SourceSpan;
use serde::{Deserialize, Serialize};

use crate::{Name, Refer, SrcRef, SrcReferrer};

/// µcad identifier
#[derive(Default, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Identifier(pub Refer<Name>);

impl SrcReferrer for Identifier {
    fn src_ref(&self) -> SrcRef {
        self.0.src_ref
    }
}

impl Identifier {
    /// Make empty (invalid) id
    pub fn none() -> Self {
        Self(Refer::none("".into()))
    }

    /// Check if id shall be ignored when warn about unused symbols
    pub fn ignore(&self) -> bool {
        self.0.starts_with("_")
    }

    /// Make empty (invalid) id
    pub fn no_ref(id: &str) -> Self {
        Self(Refer::none(id.into()))
    }

    /// Get the value of the identifier
    pub fn id(&self) -> &Name {
        &self.0.value
    }

    pub fn as_str(&self) -> &str {
        self.id().as_str()
    }

    /// Return first character of the identifier.
    pub fn short_id(&self) -> ShortId {
        let parts = self
            .0
            .value
            .split("_")
            .map(|part| {
                part.chars()
                    .next()
                    .expect("cannot shorten empty Identifier")
            })
            .map(|p| p.to_compact_string())
            .collect::<Vec<_>>()
            .join("_")
            .to_compact_string();

        ShortId(parts)
    }

    /// Return number of identifiers in name
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return if name is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if this is a valid identifier (contains only `A`-`Z`, `a`-`z` or `_`).
    pub fn is_valid(&self) -> bool {
        let str = self.0.as_str();

        // Check if is empty.
        let Some(start) = str.chars().next() else {
            return false;
        };

        (start == '_' || start.is_ascii_alphabetic())
            && str.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Detect if the identifier matches a certain case.
    pub fn detect_case(&self) -> Case {
        let s = &self.0.value;

        if s.is_empty() {
            return Case::Invalid;
        }

        if s.len() == 1 {
            let c = s.chars().next().expect("At least one char");
            if c.is_ascii_uppercase() {
                return Case::UpperSingleChar;
            } else {
                return Case::Invalid;
            }
        }

        let has_underscore = s.contains('_');

        if has_underscore {
            if s.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                return Case::UpperSnake;
            } else if s.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                return Case::LowerSnake;
            } else {
                return Case::Invalid;
            }
        } else {
            // Must be PascalCase: starts with uppercase and contains no underscores
            let mut chars = s.chars();
            if let Some(first) = chars.next()
                && first.is_ascii_uppercase()
                && chars.all(|c| c.is_ascii_alphanumeric())
            {
                return Case::Pascal;
            }
        }

        Case::Invalid
    }
}

/// A case for an identifier.
#[derive(Debug, PartialEq, Eq)]
pub enum Case {
    /// PascalCase
    Pascal,
    /// lower_snake_case
    LowerSnake,
    /// UPPER_SNAKE_CASE
    UpperSnake,
    /// A
    UpperSingleChar,
    /// Invalid.
    Invalid,
}

/// Shortened identifier
#[derive(Deref)]
pub struct ShortId(CompactString);

impl PartialEq<Identifier> for ShortId {
    fn eq(&self, other: &Identifier) -> bool {
        self.0 == *other
    }
}

impl From<Identifier> for SourceSpan {
    fn from(value: Identifier) -> Self {
        value.src_ref().into()
    }
}

impl From<Name> for Identifier {
    fn from(name: Name) -> Self {
        Identifier(Refer::none(name))
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        let identifier = Identifier::no_ref(value);
        assert!(identifier.is_valid(), "Invalid identifier: {identifier}");
        identifier
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        let identifier = Identifier::no_ref(&value);
        assert!(identifier.is_valid());
        identifier
    }
}

impl<'a> From<&'a Identifier> for &'a str {
    fn from(value: &'a Identifier) -> Self {
        &value.0
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "<NO ID>")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl PartialEq<str> for Identifier {
    fn eq(&self, other: &str) -> bool {
        *self.0 == other
    }
}

/// A list of identifiers
///
/// Used e.g. for multiple variable declarations.
/// Cannot contain duplicates.
#[derive(Default, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct IdentifierList(pub Refer<Vec<Identifier>>);

impl FromIterator<Identifier> for IdentifierList {
    fn from_iter<T: IntoIterator<Item = Identifier>>(iter: T) -> Self {
        let v: Vec<_> = iter.into_iter().collect();
        let src_ref = SrcRef::merge_all(v.iter());
        Self(Refer::new(v, src_ref))
    }
}

impl std::fmt::Display for IdentifierList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut sorted = self.0.clone();
        sorted.sort();
        write!(
            f,
            "{}",
            sorted
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl std::iter::IntoIterator for IdentifierList {
    type Item = Identifier;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Allow borrowing `Identifier` as `str`
impl std::borrow::Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        &self.0.value
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0.value
    }
}

#[test]
fn identifier_comparison() {
    use crate::{LineCol, SrcRef};
    use microcad_hash::HashId;

    // same id but different src refs
    let id1 = Identifier::no_ref("x");
    let id2 = Identifier(Refer::new(
        "x".into(),
        SrcRef::new(&(0..5), LineCol { line: 0, col: 1 }, HashId::default()),
    ));

    // shall be equal
    assert!(id1 == id2);
}

#[test]
fn identifier_hash() {
    use microcad_hash::HashId;

    use crate::{LineCol, SrcRef};
    use std::hash::{Hash, Hasher};

    // same id but different src refs
    let id1 = Identifier(Refer::none("x".into()));
    let id2 = Identifier(Refer::new(
        "x".into(),
        SrcRef::new(&(0..5), LineCol { line: 0, col: 1 }, HashId::default()),
    ));

    let mut hasher = std::hash::DefaultHasher::new();
    id1.hash(&mut hasher);
    let hash1 = hasher.finish();
    let mut hasher = std::hash::DefaultHasher::new();
    id2.hash(&mut hasher);

    let hash2 = hasher.finish();

    // shall be equal
    assert_eq!(hash1, hash2);
}

#[test]
fn identifier_case() {
    let detect_case = |s| -> Case { Identifier::no_ref(s).detect_case() };

    assert_eq!(detect_case("PascalCase"), Case::Pascal);
    assert_eq!(detect_case("lower_snake_case"), Case::LowerSnake);
    assert_eq!(detect_case("UPPER_SNAKE_CASE"), Case::UpperSnake);
    assert_eq!(detect_case("notValid123_"), Case::Invalid);
    assert_eq!(detect_case(""), Case::Invalid);
    assert_eq!(detect_case("A"), Case::UpperSingleChar); // New case
    assert_eq!(detect_case("z"), Case::Invalid); // lowercase single letter
    assert_eq!(detect_case("_"), Case::Invalid); // only underscore
    assert_eq!(detect_case("a_b"), Case::LowerSnake);
    assert_eq!(detect_case("A_B"), Case::UpperSnake);

    println!("All tests passed.");
}

#[test]
fn test_short_identifiers() {
    fn test(id: &str) -> String {
        Identifier::from(id).short_id().to_string()
    }

    assert_eq!(test("weather_thermal_function"), "w_t_f");
    assert_eq!(test("width"), "w");
    assert_eq!(test("WeatherThermal_Function"), "W_F");
}
