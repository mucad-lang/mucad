// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The old module, declared deprecated.
#[deprecated(since = "0.6.0")]
pub mod old; // Loads stability_file_module/old.mu

/// The new module, experimental by its file.
pub mod new; // Loads stability_file_module/new.mu
