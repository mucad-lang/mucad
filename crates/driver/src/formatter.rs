// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad code formatter

use crate::prelude as mu;

#[derive(Debug, Default)]
pub struct Formatter {
    config: mu::FormatConfig,
}

impl Formatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: mu::FormatConfig) -> Self {
        self.config = config;
        self
    }
}

impl Formatter {
    /// Formats a `Source` and return a formatted source and a flag if formatting has happened.
    pub fn format(&self, source: impl Into<mu::Source>) -> Result<mu::Formatted, mu::Diagnostics> {
        mu::format(source, &self.config).map_err(|issues| issues.into_diagnostics())
    }
}
