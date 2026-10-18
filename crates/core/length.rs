// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad Length type.

use super::Scalar;
use derive_more::{Deref, DerefMut};

/// A length in millimeters.
#[derive(Clone, Debug, Copy, Default, Deref, DerefMut)]
pub struct Length(pub Scalar);

impl Length {
    /// Return a new length from millimeters.
    pub fn mm(mm: Scalar) -> Self {
        Self(mm)
    }
}
