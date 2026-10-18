// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Model output type.

use mucad_lang_base::element::WorkbenchKind;
use serde::{Deserialize, Serialize};

/// The output type of the [`crateModel`].
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum ModelType {
    /// The output type has not yet been determined.
    #[default]
    NotDetermined,
    /// The [`Model`] outputs a 2d geometry.
    Geometry2D,
    /// The [`Model`] outputs a 3d geometry.
    Geometry3D,
    /// The [`Model`] contains both 2D and 3D geometry.
    Any,
}

impl ModelType {
    /// Merge this output type with another.
    pub fn merge(&self, other: &Self) -> ModelType {
        match (self, other) {
            (ModelType::NotDetermined, output_type) => *output_type,
            (ModelType::Geometry2D, ModelType::NotDetermined)
            | (ModelType::Geometry2D, ModelType::Geometry2D)
            | (ModelType::Geometry3D, ModelType::NotDetermined)
            | (ModelType::Geometry3D, ModelType::Geometry3D) => *self,
            (ModelType::Geometry2D, ModelType::Geometry3D)
            | (ModelType::Geometry3D, ModelType::Geometry2D)
            | (ModelType::Geometry2D, ModelType::Any)
            | (ModelType::Geometry3D, ModelType::Any)
            | (ModelType::Any, _) => ModelType::Any,
        }
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::NotDetermined => "Undetermined",
                Self::Geometry2D => "2D",
                Self::Geometry3D => "3D",
                Self::Any => "2D/3D",
            }
        )
    }
}

impl From<WorkbenchKind> for ModelType {
    fn from(kind: WorkbenchKind) -> Self {
        match kind {
            WorkbenchKind::Sketch => Self::Geometry2D,
            WorkbenchKind::Part => Self::Geometry3D,
            WorkbenchKind::Op => Self::NotDetermined,
        }
    }
}
