// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Viewer bevy systems

use std::marker::PhantomData;

use bevy::{
    asset::Assets,
    ecs::system::{Commands, Res, ResMut},
    mesh::{Mesh, Mesh3d},
    pbr::StandardMaterial,
    prelude::*,
};
use bevy_mod_outline::{OutlineMode, OutlineVolume};

use crate::stdin::StdinMessageReceiver;
use crate::view_model::ModelViewState;
use crate::*;
use crate::{
    processor::{ProcessorRequest, ProcessorResponse, registry},
    view_model::ViewerEvent,
};

use mucad_driver::prelude as mu;

/// Whether a kind of watch event is relevant for compilation.
fn is_relevant_event_kind(kind: &notify::EventKind) -> bool {
    match kind {
        notify::EventKind::Any => false,
        notify::EventKind::Access(_) => false,
        notify::EventKind::Create(_) => true,
        notify::EventKind::Modify(kind) => match kind {
            notify::event::ModifyKind::Any => true,
            notify::event::ModifyKind::Data(_) => true,
            notify::event::ModifyKind::Metadata(_) => true,
            notify::event::ModifyKind::Name(_) => true,
            notify::event::ModifyKind::Other => false,
        },
        notify::EventKind::Remove(_) => true,
        notify::EventKind::Other => false,
    }
}

/// Start up the processor.
///
/// Sends an initialize request to the processor and handles input.
pub fn initialize_processor(mut state: ResMut<crate::view_model::ViewModel>) {
    state
        .processor
        .send_request(ProcessorRequest::Initialize {
            config: state.config.clone(),
        })
        .expect("No error");

    use crate::plugin::MicrocadPluginInput;
    let reload_delay = state.config.reload_delay;

    let mut requests = Vec::new();

    match &mut state.input {
        Some(MicrocadPluginInput::File {
            path,
            symbol: _,
            line,
            last_modified,
        }) => {
            let flag_clone = last_modified.clone();
            let path = path.clone();
            requests.push(ProcessorRequest::CompileFile(path.clone()));
            requests.push(ProcessorRequest::SetLineNumber(*line));

            // Run file watcher thread.
            std::thread::spawn(move || -> ! {
                use notify::{RecursiveMode, Watcher};

                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = notify::recommended_watcher(tx).expect("Some watcher");
                watcher
                    .watch(&path, RecursiveMode::NonRecursive)
                    .expect("No error");

                log::info!("Watching external file: {}", path.display());

                loop {
                    if let Ok(Ok(event)) = rx.recv_timeout(reload_delay)
                        && is_relevant_event_kind(&event.kind)
                        && let Ok(meta) = std::fs::metadata(&path)
                        && let Ok(modified) = meta.modified()
                    {
                        log::info!("Modified");
                        *flag_clone.lock().expect("Lock") = Some(modified);
                        watcher
                            .watch(&path, RecursiveMode::NonRecursive)
                            .expect("No error");
                    }
                }
            });
        }
        Some(MicrocadPluginInput::Stdin(stdin)) => {
            log::info!("Run viewer in stdin remote controlled mode.");
            *stdin = Some(StdinMessageReceiver::run());
        }
        _ => { /* Do nothing */ }
    }

    requests
        .into_iter()
        .for_each(|request| state.processor.send_request(request).expect("No error"));
}

pub fn file_reload(state: ResMut<crate::view_model::ViewModel>) {
    use crate::plugin::MicrocadPluginInput::*;

    match &state.input {
        Some(File {
            path,
            last_modified,
            ..
        }) => {
            let mut last_modified_lock = last_modified.lock().expect("Lock");
            if let Some(last_modified) = *last_modified_lock
                && let Ok(elapsed) = last_modified.elapsed()
                && elapsed > state.config.reload_delay
            {
                state
                    .processor
                    .send_request(ProcessorRequest::CompileFile(path.to_path_buf()))
                    .expect("No error");
                log::info!("Changed file");

                // Reset so we don’t reload again
                *last_modified_lock = None;
            }
        }
        _ => { /* Do nothing */ }
    }
}

/// A marker for the contour edges of a mesh, e.g. to toggle their visibility.
#[derive(Component)]
pub struct ContourEdges;

/// This system handles responses coming from the processor and fills the Bevy command pipeline.
pub fn handle_processor_responses(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut model_view_states: ResMut<Assets<ModelViewState>>,
    mut view_model: ResMut<ViewModel>,
    mut events: MessageWriter<ViewerEvent>,
) {
    let mut entities = Vec::new();
    let mut ground_radius = mu::core::Length::default();

    for response in view_model.processor.response_receiver.try_iter() {
        match response {
            ProcessorResponse::RemoveModelInstances(uuids) => uuids.iter().for_each(|uuid| {
                model_view_states.remove(*uuid);
                materials.remove(*uuid);
            }),
            ProcessorResponse::NewMeshAsset(uuid, mesh, edges) => {
                log::info!("New mesh: {uuid}");
                let _ = meshes.insert(uuid, mesh);
                if let Some(edges) = edges {
                    let _ = meshes.insert(registry::edge_mesh_uuid(uuid), edges);
                }
            }
            ProcessorResponse::NewModelInfo(uuid, info) => {
                log::info!("New model info: {uuid}");
                let _ = model_view_states.insert(uuid, ModelViewState::new(info, &view_model));
            }
            ProcessorResponse::UpdateMaterials(uuids) => {
                uuids.iter().for_each(|uuid| {
                    let view_state = model_view_states.get(*uuid).expect("Model info");
                    let _ = materials.insert(*uuid, view_state.generate_material());
                });
            }
            ProcessorResponse::SpawnModelInstances(uuids) => {
                let _ = materials.insert(
                    crate::material::EDGE_MATERIAL_UUID,
                    crate::material::create_edge_material(
                        view_model.config.theme.primary.to_bevy(),
                    ),
                );
                // A model is spawned as one entity per mesh, which share the model's material.
                for (uuid, view_state) in uuids
                    .iter()
                    .filter_map(|uuid| Some((*uuid, model_view_states.get(*uuid)?)))
                {
                    log::info!("Spawn model: {uuid}");
                    ground_radius =
                        mu::core::Length::mm(ground_radius.max(view_state.info().ground_radius.0));

                    for mesh_uuid in &view_state.info().mesh_uuids {
                        let mut entity = commands.spawn((
                            Mesh3d(Handle::Uuid(*mesh_uuid, PhantomData)),
                            MeshMaterial3d::<StandardMaterial>(Handle::Uuid(uuid, PhantomData)),
                            view_state.info().transform,
                            view_state.outline_volume.clone(),
                            OutlineMode::FloodFlat,
                            view_state.clone(),
                        ));
                        // The contour edges are a child of their mesh's entity: they share its
                        // transform and are despawned with it, but can be hidden on their own.
                        let edge_mesh_uuid = registry::edge_mesh_uuid(*mesh_uuid);
                        if meshes.contains(AssetId::<Mesh>::Uuid {
                            uuid: edge_mesh_uuid,
                        }) {
                            entity.with_child((
                                Mesh3d(Handle::Uuid(edge_mesh_uuid, PhantomData)),
                                MeshMaterial3d::<StandardMaterial>(Handle::Uuid(
                                    crate::material::EDGE_MATERIAL_UUID,
                                    PhantomData,
                                )),
                                ContourEdges,
                                bevy::picking::Pickable::IGNORE,
                            ));
                        }
                        entities.push(entity.id());
                    }
                }
            }
            ProcessorResponse::StateChanged(state) => {
                events.write(ViewerEvent::ProcessingStateChanged(state));
            }
        }

        if view_model.processor.response_receiver.is_empty() {
            break;
        }
    }

    if !entities.is_empty() {
        // Despawn all entities to remove them from the scene
        for entity in &view_model.scene.model_entities {
            commands.entity(*entity).despawn();
        }

        view_model.scene.model_entities = entities;
        events.write(ViewerEvent::ChangeGroundRadius(ground_radius));
    }
}

/// A system that draws hit indicators for every pointer.
pub fn handle_pick_event(
    pointers: Query<&bevy::picking::pointer::PointerInteraction>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(
        &ModelViewState,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut OutlineVolume,
    )>,
    mut events: MessageWriter<ViewerEvent>,
) {
    for (entity, _) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
    {
        match query.get_mut(*entity) {
            Ok((view_state, ref mut _material, ref mut _outline)) => {
                if buttons.just_pressed(MouseButton::Left) {
                    events.write(ViewerEvent::SelectOne(view_state.info().model_uuid));
                }
            }
            // No Hit was found..
            Err(_) => {
                if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
                    events.write(ViewerEvent::ClearSelection);
                }
            }
        }
    }
}
