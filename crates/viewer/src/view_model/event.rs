// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad State event.

use bevy::{
    asset::{Assets, uuid::Uuid},
    camera::{Camera, Projection},
    ecs::{
        message::{Message, MessageReader},
        query::With,
        system::{Query, ResMut},
    },
    pbr::{MeshMaterial3d, StandardMaterial},
    window::Window,
};

use mucad_driver::prelude as mu;

use crate::{
    ViewModel, material,
    processor::ProcessingState,
    view_model::{Cursor, ModelViewState},
};

/// An event that is fired when the view model changes in some way.
#[derive(Message)]
pub enum ViewerEvent {
    /// The ground radius has changed.
    ChangeGroundRadius(mu::core::Length),
    /// All models have been selected.
    SelectAll,
    /// Clear the selection.
    ClearSelection,
    /// Select a model.
    SelectOne(Uuid),
    /// Set the cursor position in a source file to highlighted.
    SetCursor(Cursor),
    /// Zoom to fit.
    ZoomToFit,
    /// The processing state has changed (e.g. required to show the progress bar).
    ProcessingStateChanged(ProcessingState),
}

impl ViewerEvent {
    fn for_each_view_state(
        f: impl Fn(&Uuid, &mut ModelViewState),
        view_states: &mut Assets<ModelViewState>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        view_states
            .iter_mut()
            .for_each(|(asset_id, view_state)| match asset_id {
                bevy::asset::AssetId::Uuid { uuid } => {
                    // Generate new material.
                    f(&uuid, view_state);
                    let material = materials.get_mut(uuid).expect("Must have material");
                    *material = view_state.generate_material();
                }
                _ => unreachable!(),
            });
    }
}

/// Handler for viewer events.
#[allow(clippy::too_many_arguments)]
pub fn handle_viewer_event(
    mut grid_materials: ResMut<Assets<material::Grid>>,
    mat_query: Query<&mut MeshMaterial3d<material::Grid>>,
    windows: Query<&Window>,
    mut projection: Query<&mut Projection, With<Camera>>,
    mut state: ResMut<ViewModel>,
    mut view_states: ResMut<Assets<ModelViewState>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: MessageReader<ViewerEvent>,
) {
    for event in events.read() {
        match event {
            ViewerEvent::ChangeGroundRadius(radius) => {
                if let Some(grid) = state.scene.grid_entity
                    && let Ok(material) = mat_query.get(grid)
                    && let Some(material) = grid_materials.get_mut(material)
                {
                    state.scene.radius = **radius as f32;
                    material.parameters.radius = **radius as f32;
                }
            }
            ViewerEvent::SelectAll => {
                ViewerEvent::for_each_view_state(
                    |_, view_state| {
                        view_state.is_selected = false;
                    },
                    view_states.as_mut(),
                    materials.as_mut(),
                );
            }
            ViewerEvent::ClearSelection => {
                ViewerEvent::for_each_view_state(
                    |_, view_state| {
                        view_state.is_selected = false;
                    },
                    view_states.as_mut(),
                    materials.as_mut(),
                );
            }
            ViewerEvent::SelectOne(selected_uuid) => {
                ViewerEvent::for_each_view_state(
                    |uuid, view_state| {
                        view_state.is_selected = uuid == selected_uuid;
                    },
                    view_states.as_mut(),
                    materials.as_mut(),
                );
            }
            ViewerEvent::SetCursor(_) => ViewerEvent::for_each_view_state(
                |_, _| todo!(),
                view_states.as_mut(),
                materials.as_mut(),
            ),
            ViewerEvent::ZoomToFit => {
                let Ok(mut projection) = projection.single_mut() else {
                    return;
                };
                let Ok(window) = windows.single() else {
                    return;
                };
                crate::scene::zoom_to_fit(projection.as_mut(), window);
            }
            ViewerEvent::ProcessingStateChanged(processing_state) => {
                state.processing_state = processing_state.clone();
            }
        }
    }
}
