// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Model Registry.

use bevy::asset::uuid::Uuid;
use mucad_driver::prelude as mu;

const MODEL_UUID_SEED: u64 = 0xDEAD_BEEF_DEED_BEAF;

const MESH_UUID_SEED: u64 = 0x4321_4321_4321_4321;

/// Generate a Uuid for the model info of a geometry node, which is unique within its geometry tree.
pub fn generate_model_uuid(node: mu::render::GeometryNodeRef) -> Uuid {
    Uuid::from_u64_pair(MODEL_UUID_SEED, usize::from(node.id) as u64)
}

/// Generate a Uuid for the mesh at `index` of the geometry outputs with `hash`.
pub fn generate_mesh_uuid(hash: mu::HashId, index: usize) -> Uuid {
    Uuid::from_u64_pair(MESH_UUID_SEED ^ index as u64, hash.raw())
}

/// The instance registry keeps track of all Uuid that already have been rendered, so we don't have have re-render things.
#[derive(Default)]
pub struct InstanceRegistry {
    /// UUIDs of the meshes of geometry outputs by their hash.
    mesh_uuids: mu::HashMap<mu::HashId, Vec<Uuid>>,
    /// UUIDs of all model infos.
    model_uuids: mu::HashSet<Uuid>,
}

impl InstanceRegistry {
    /// The UUIDs of the meshes of the geometry outputs with `hash`, if they have been created.
    pub fn mesh_uuids(&self, hash: &mu::HashId) -> Option<&Vec<Uuid>> {
        self.mesh_uuids.get(hash)
    }

    pub fn contains_model(&self, uuid: &Uuid) -> bool {
        self.model_uuids.contains(uuid)
    }

    pub fn insert_mesh_uuids(&mut self, hash: mu::HashId, uuids: Vec<Uuid>) {
        self.mesh_uuids.insert(hash, uuids);
    }

    pub fn insert_model(&mut self, uuid: Uuid) {
        self.model_uuids.insert(uuid);
    }

    pub fn fetch_model_uuids(&self) -> Vec<Uuid> {
        self.model_uuids.iter().cloned().collect()
    }

    pub fn clear_model_uuids(&mut self) {
        self.model_uuids.clear()
    }
}
