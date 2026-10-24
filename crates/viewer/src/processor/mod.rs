// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad Viewer processor.

mod context;
mod model_info;
mod processing;
mod registry;
mod request;
mod systems;

pub use crate::processor::{
    context::ProcessingState, model_info::ModelInfo, request::ProcessorRequest,
};

use crate::processor::{context::ProcessorContext, processing::Processor};
use bevy::{
    app::{Plugin, Startup, Update},
    asset::uuid::Uuid,
    mesh::Mesh,
};

use crossbeam::channel::{Receiver, Sender};
use miette::IntoDiagnostic;

/// A processor response.
///
/// Contains the geometry to rendered.
pub enum ProcessorResponse {
    /// Remove model instances from the registry (this mean the model will rerendered in the next render call).
    RemoveModelInstances(Vec<Uuid>),
    /// Create a new bevy mesh asset for a model with the mesh of its contour edges, if any.
    ///
    /// The edges are stored with the uuid of [`registry::edge_mesh_uuid`].
    NewMeshAsset(Uuid, Mesh, Option<Mesh>),
    /// Create a new model info.
    NewModelInfo(Uuid, ModelInfo),
    /// Update the materials for models.
    UpdateMaterials(Vec<Uuid>),
    /// Model instances to be spawned for a frame.
    SpawnModelInstances(Vec<Uuid>),
    /// This response is sent each time a the processing has changed.
    StateChanged(ProcessingState),
}

/// The processor interface.
pub struct ProcessorInterface {
    request_sender: Sender<ProcessorRequest>,
    /// The receiver for responses.
    pub response_receiver: Receiver<ProcessorResponse>,
}

impl ProcessorInterface {
    /// Send request.
    pub fn send_request(&self, request: ProcessorRequest) -> miette::Result<()> {
        self.request_sender.send(request).into_diagnostic()
    }

    /// Run the processing thread and create interface.
    pub fn run() -> Self {
        let (request_sender, request_receiver) = crossbeam::channel::unbounded();
        let (response_sender, response_receiver) = crossbeam::channel::unbounded();

        std::thread::spawn(move || {
            let mut processor = Processor {
                context: ProcessorContext::default(),
                request_receiver,
                response_sender,
            };

            loop {
                if let Ok(request) = processor.request_receiver.recv()
                    && let Ok(responses) = processor.handle_request(request)
                {
                    for response in responses {
                        processor.response_sender.send(response).expect("No error");
                    }
                }
            }
        });

        Self {
            request_sender,
            response_receiver,
        }
    }
}

/// The processor bevy plugin.
pub struct ProcessorPlugin;

impl Plugin for ProcessorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, systems::initialize_processor)
            .add_systems(Update, systems::handle_processor_responses)
            .add_systems(Update, systems::file_reload)
            .add_systems(Update, systems::handle_pick_event);
    }
}
