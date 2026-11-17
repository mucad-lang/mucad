// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Introduced with the name of the version.
#[introduced(version = "0.6.0")]
pub const NAMED = 1;

/// Introduced without the name of the version.
#[experimental]
#[introduced("0.7.0")]
pub const POSITIONAL = 2;
