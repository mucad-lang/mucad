// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bezier implementation

pub use bezier_rs::{Bezier, Identifier, TValue};

/// An empty id type for use in tests
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct EmptyId;

impl Identifier for EmptyId {
    fn new() -> Self {
        Self
    }
}

/// Subpath with empty id
pub type Subpath = bezier_rs::Subpath<EmptyId>;
