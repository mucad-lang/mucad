// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! An ID to reference an item in an external library.

use derive_more::Display;
use indextree::NodeId;
use serde::{Deserialize, Serialize};

use crate::LibraryId;

#[derive(Debug, Display, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[display("{lib_id}@{id}")]
pub struct ExternalId {
    pub lib_id: LibraryId,
    pub id: NodeId,
}
