// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crossbeam::channel::{Receiver, Sender};
use mucad_driver::prelude::{self as mu, traits::RenderDiag};
use miette::IntoDiagnostic;

use crate::{
    processor::{
        ModelInfo, ProcessorRequest, ProcessorResponse,
        context::{ProcessingState, ProcessorContext},
    },
    to_bevy::ToBevyMesh,
};

/// The processor is responsible for generating bevy meshes and commands sent to the view.
///
/// The processor itself runs in a separate thread and can be controlled
/// via [`ProcessorInterface`] by sending requests and handling the corresponding responses.
pub struct Processor {
    /// The state of the processor.
    pub context: ProcessorContext,

    /// Requests.
    pub request_receiver: Receiver<ProcessorRequest>,

    /// Output responses.
    pub response_sender: Sender<ProcessorResponse>,
}

impl Processor {
    fn state_change(&mut self, state: ProcessingState) {
        self.context.state = state.clone();
        self.response_sender
            .send(ProcessorResponse::StateChanged(state))
            .expect("No error")
    }

    /// Compile the source, render its model and generate the responses for the view.
    pub(crate) fn compile(&mut self, source: mu::Source) -> miette::Result<Vec<ProcessorResponse>> {
        self.state_change(ProcessingState::Busy(0.0));

        let (tx, rx) = std::sync::mpsc::channel();
        let sender = self.response_sender.clone();
        std::thread::spawn(move || {
            while let Ok(progress) = rx.recv() {
                sender
                    .send(ProcessorResponse::StateChanged(ProcessingState::Busy(
                        progress,
                    )))
                    .expect("No error");
            }
        });
        self.context.renderer.progress_tx = Some(tx);

        let compiler = &mut self.context.compiler;
        let responses = match compiler.compile(source) {
            Ok((model_tree, diags)) if !diags.has_errors() => {
                eprintln!("{}", compiler.render_diag_to_string(&diags));
                self.respond(&model_tree)
            }
            Ok((_, diags)) | Err(diags) => {
                eprintln!("{}", compiler.render_diag_to_string(&diags));
                self.state_change(ProcessingState::Error);
                Ok(vec![])
            }
        }?;

        // Remove unused cache items.
        if let Some(cache) = &self.context.renderer.cache {
            log::info!("Render cache");
            cache.with_write(|cache| cache.garbage_collection());
        }

        Ok(responses)
    }

    /// Handle processor request.
    pub(crate) fn handle_request(
        &mut self,
        request: ProcessorRequest,
    ) -> miette::Result<Vec<ProcessorResponse>> {
        match request {
            ProcessorRequest::Initialize { config } => {
                self.context.compiler = mu::Compiler::from_config(&mu::DriverConfig {
                    search_paths: config.search_paths.clone(),
                    ..Default::default()
                });
                self.context.renderer.resolution = mu::RenderResolution {
                    linear: config.render_resolution,
                };
                self.context.theme = config.theme;
                self.context.initialized = true;
                self.context.state = ProcessingState::Idle;
                Ok(vec![])
            }
            ProcessorRequest::CompileFile(path) => {
                self.compile(mu::Source::load(path).into_diagnostic()?)
            }
            ProcessorRequest::CompileSource {
                path,
                name: _name,
                source,
            } => self.compile(match path {
                Some(path) => mu::Source::new(path, source),
                None => mu::Source::from(source.as_str()),
            }),
            ProcessorRequest::Export { .. } => todo!(),
            ProcessorRequest::SetLineNumber(line_number) => {
                self.state_change(ProcessingState::Busy(0.0));
                self.context.line_number = line_number;
                Ok(vec![])
                //self.respond()
            }
            _ => unreachable!(),
        }
    }

    /// Render the model tree, update the model instances and generate processor responses.
    fn respond(&mut self, model_tree: &mu::ModelTree) -> miette::Result<Vec<ProcessorResponse>> {
        // The geometry tree is temporary, its geometry is kept in the render cache.
        // Render errors are shown, the geometry rendered despite them is shown too.
        let geometry_tree = match self.context.renderer.render(model_tree) {
            Ok((geometry_tree, issues)) => {
                if !issues.is_empty() {
                    let diags = issues.into_diagnostics();
                    eprintln!("{}", self.context.renderer.render_diag_to_string(&diags));
                }
                geometry_tree
            }
            Err(issues) => {
                let diags = issues.into_diagnostics();
                eprintln!("{}", self.context.renderer.render_diag_to_string(&diags));
                self.state_change(ProcessingState::Error);
                return Ok(vec![]);
            }
        };

        let mut responses = Vec::new();
        responses.push(ProcessorResponse::RemoveModelInstances(
            self.context.instance_registry.fetch_model_uuids(),
        ));

        self.context.instance_registry.clear_model_uuids();
        self.generate_responses(geometry_tree.root(), model_tree, &mut responses);
        log::info!("{} responses", responses.len());

        responses.push(ProcessorResponse::UpdateMaterials(
            self.context.instance_registry.fetch_model_uuids(),
        ));

        responses.push(ProcessorResponse::SpawnModelInstances(
            self.context.instance_registry.fetch_model_uuids(),
        ));

        self.state_change(ProcessingState::Idle);

        Ok(responses)
    }

    /// Generate meshes and model infos for the workpieces with geometry, starting at `node`.
    ///
    /// The geometry of a workpiece contains the geometry of its children,
    /// so children are only visited if a node has no geometry.
    fn generate_responses(
        &mut self,
        node: mu::render::GeometryNodeRef,
        model_tree: &mu::ModelTree,
        responses: &mut Vec<ProcessorResponse>,
    ) {
        use mu::types::{ModelType, model::Element};

        if !matches!(
            node.output_type,
            ModelType::Geometry2D | ModelType::Geometry3D
        ) {
            return;
        }

        let model = node.model(model_tree);
        let is_workpiece = matches!(
            model.get().element,
            Element::Workpiece(_) | Element::BuiltinWorkpiece(_)
        );
        if !is_workpiece || node.outputs.is_empty() {
            node.children()
                .for_each(|child| self.generate_responses(child, model_tree, responses));
            return;
        }

        // Add new mesh assets, if the geometry has no meshes yet.
        let mesh_uuids = match self.context.instance_registry.mesh_uuids(&node.hash) {
            Some(mesh_uuids) => mesh_uuids.clone(),
            None => {
                let mesh_uuids: Vec<_> = node
                    .outputs
                    .iter()
                    .flat_map(|output| match &output.geometry {
                        mu::core::Geometry::Geometry2D(geometry) => geometry.to_bevy_mesh_default(),
                        mu::core::Geometry::Geometry3D(geometry) => geometry.to_bevy_mesh(30.0),
                    })
                    .enumerate()
                    .map(|(index, mesh)| {
                        let uuid = crate::processor::registry::generate_mesh_uuid(node.hash, index);
                        responses.push(ProcessorResponse::NewMeshAsset(uuid, mesh));
                        uuid
                    })
                    .collect();
                self.context
                    .instance_registry
                    .insert_mesh_uuids(node.hash, mesh_uuids.clone());
                mesh_uuids
            }
        };

        let model_info = ModelInfo::new(node, model, mesh_uuids);
        if !self
            .context
            .instance_registry
            .contains_model(&model_info.model_uuid)
        {
            self.context
                .instance_registry
                .insert_model(model_info.model_uuid);
            responses.push(ProcessorResponse::NewModelInfo(
                model_info.model_uuid,
                model_info,
            ));
        }
    }
}
