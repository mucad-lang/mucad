// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export attribute.

use derive_more::Display;
use mucad_lang_base::Name;
use serde::{Deserialize, Serialize};

use crate::{Value, tuple};

/// Export attribute, e.g. `#[export: "output.svg"]`.
#[derive(Clone, Debug, Display, PartialEq, Hash, Serialize, Deserialize)]
#[display("path = {}, id = {}", path.display(), id)]
pub struct ExportCommand {
    /// Filename.
    pub path: std::path::PathBuf,
    /// Exporter id
    pub id: Name,
}

impl From<ExportCommand> for Value {
    fn from(export: ExportCommand) -> Self {
        Value::from(tuple!(
            path = export.path.display().to_string(),
            id = export.id
        ))
    }
}
