// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the geometry processor, which runs without a window.

use mucad_viewer::{
    Config,
    processor::{ProcessorInterface, ProcessorRequest, ProcessorResponse},
};

/// A processor, which finds the standard library of this repository.
fn processor() -> ProcessorInterface {
    let processor = ProcessorInterface::run();
    processor
        .send_request(ProcessorRequest::Initialize {
            config: Config {
                // The standard library of this repository (relative to `crates/viewer`).
                search_paths: vec!["../std/lib".into()],
                ..Default::default()
            },
        })
        .expect("No error");
    processor
}

/// The responses for the compilation of `source` until the model instances are spawned.
fn responses(source: &str) -> Vec<ProcessorResponse> {
    let processor = processor();
    processor
        .send_request(ProcessorRequest::CompileSource {
            path: None,
            name: None,
            source: source.into(),
        })
        .expect("No error");

    let mut responses = Vec::new();
    while let Ok(response) = processor
        .response_receiver
        .recv_timeout(std::time::Duration::from_secs(30))
    {
        let spawned = matches!(response, ProcessorResponse::SpawnModelInstances(_));
        responses.push(response);
        if spawned {
            break;
        }
    }
    responses
}

/// Each workpiece with geometry gets a model info with its meshes.
#[test]
fn process_workpieces() {
    let responses = responses("Circle(radius = 1mm);\nRect(2mm).translate(x = 5mm);");

    let meshes = responses
        .iter()
        .filter(|response| matches!(response, ProcessorResponse::NewMeshAsset(..)))
        .count();
    let model_infos: Vec<_> = responses
        .iter()
        .filter_map(|response| match response {
            ProcessorResponse::NewModelInfo(_, info) => Some(info),
            _ => None,
        })
        .collect();

    assert_eq!(model_infos.len(), 2);
    assert_eq!(meshes, 2);
    assert!(model_infos.iter().all(|info| info.mesh_uuids.len() == 1));
    assert!(matches!(
        responses.last(),
        Some(ProcessorResponse::SpawnModelInstances(uuids)) if uuids.len() == 2
    ));
}

/// A source with errors spawns no models.
#[test]
fn process_errors() {
    let processor = processor();
    processor
        .send_request(ProcessorRequest::CompileSource {
            path: None,
            name: None,
            source: "Unknown();".into(),
        })
        .expect("No error");

    let responses: Vec<_> = processor
        .response_receiver
        .iter()
        .take_while(|response| {
            !matches!(
                response,
                ProcessorResponse::StateChanged(mucad_viewer::processor::ProcessingState::Error)
            )
        })
        .collect();
    assert!(
        !responses
            .iter()
            .any(|response| matches!(response, ProcessorResponse::NewModelInfo(..)))
    );
}
