// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// Line and column within a source code file
#[derive(Clone, Copy, Hash, Debug, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct LineCol {
    /// Line number (0..)
    pub line: u32,
    /// Column number (1..)
    pub col: u32,
}

impl std::fmt::Display for LineCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col)
    }
}
