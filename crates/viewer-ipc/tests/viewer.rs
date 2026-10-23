// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Viewer processes.

use mucad_viewer_ipc::{Viewer, ViewerCommand, ViewerProcess, ViewerRequest};

/// The µcad viewer reads the requests from stdin, hidden if the window is not shown.
#[test]
fn mucad_viewer_command() {
    let hidden = ViewerCommand::mucad_viewer(false);
    assert_eq!(hidden.args, ["--hidden", "--stay-on-top", "-v", "stdin://"]);
    assert_eq!(
        ViewerCommand::mucad_viewer(true).args,
        ["--stay-on-top", "-v", "stdin://"]
    );
}

/// Any program can be a viewer, which gets the requests as JSON lines on stdin,
/// e.g. a shell writing them into a file.
#[cfg(unix)]
#[test]
fn any_viewer_process() {
    let file = std::env::temp_dir().join(format!("mucad-viewer-ipc-{}.jsonl", std::process::id()));
    let viewer = ViewerProcess::run(
        ViewerCommand::new("sh")
            .arg("-c")
            .arg(format!("cat > '{}'", file.display())),
    );
    viewer
        .send_request(ViewerRequest::ZoomToFit)
        .expect("A request");
    viewer.send_request(ViewerRequest::Exit).expect("A request");

    // The viewer runs in its own process, so wait for the requests.
    let mut lines = Vec::new();
    for _ in 0..100 {
        lines = std::fs::read_to_string(&file)
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect();
        if lines.len() == 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let _ = std::fs::remove_file(&file);

    let requests: Vec<_> = lines
        .iter()
        .map(|line| serde_json::from_str::<ViewerRequest>(line).expect("A request"))
        .collect();
    assert!(
        matches!(
            requests[..],
            [ViewerRequest::ZoomToFit, ViewerRequest::Exit]
        ),
        "{lines:?}"
    );
}
