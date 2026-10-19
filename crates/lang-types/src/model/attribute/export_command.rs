// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export attribute.

use serde::{Deserialize, Serialize};

use crate::Value;

/// Export attribute, e.g. `#[export = "output.svg"]`.
#[derive(Clone, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub struct ExportCommand {
    /// Optional filename.
    pub path: Option<String>,
}

// Used mainly for attribute access.
impl From<ExportCommand> for Value {
    fn from(export: ExportCommand) -> Self {
        export.path.map(Value::from).unwrap_or_default()
    }
}

impl std::fmt::Display for ExportCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(path) => write!(f, "#[export = {path}]"),
            None => write!(f, "#[export]"),
        }
    }
}
