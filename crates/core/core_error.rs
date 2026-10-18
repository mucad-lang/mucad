// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core error

use thiserror::Error;

/// Core error
#[derive(Debug, Error)]
pub enum CoreError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Parse float error
    #[error("Parse float error: {0}")]
    Error(#[from] std::num::ParseFloatError),
}

/// Core result type
pub type CoreResult<T> = std::result::Result<T, CoreError>;
