// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad viewer ipc manual test

use mucad_viewer_ipc::*;

fn prompt_for_confirmation(prompt: &str) -> std::io::Result<bool> {
    loop {
        println!("{} (y/n)", prompt);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "y" => return Ok(true),
            "n" => return Ok(false),
            _ => println!("Please enter 'y' or 'n'."),
        }
    }
}

fn example_files() -> miette::Result<Vec<std::path::PathBuf>> {
    Ok(std::fs::read_dir("examples")
        .map_err(|err| miette::miette!("{err}"))?
        .filter_map(|entry| {
            // entry? inside filter_map must be handled manually
            let path = entry.ok()?.path();
            mucad_lang::resolve::is_mucad_file(&path).then_some(path)
        })
        .collect())
}

/// Test minimize/restoring window
fn test_minimize_restore() -> miette::Result<()> {
    env_logger::init();
    let viewer = ViewerProcessInterface::run(false); // Start hidden

    let mut cycle = 0;

    loop {
        viewer
            .send_request(ViewerRequest::Restore)
            .expect("Successful restore request.");
        prompt_for_confirmation("Is the window visible?")
            .expect("Window did not appear as expected.");

        viewer
            .send_request(ViewerRequest::ShowSourceCode {
                path: None,
                name: Some("Test".to_string()),
                code: format!(
                    r#"
                use std::geo2d::Text;
                Text("{cycle}", height = 10mm);
            "#
                ),
            })
            .expect("Successful show source code request.");

        prompt_for_confirmation("Is there a number?").expect("Valid source code");

        viewer
            .send_request(ViewerRequest::Minimize)
            .expect("Successful minimize request.");
        prompt_for_confirmation("Is the window hidden?").expect("Window did not hide as expected.");
        cycle += 1;

        log::info!("Show/Hide Cycle #{cycle}")
    }
}

fn test_code_from_file() -> miette::Result<()> {
    env_logger::init();
    let viewer = ViewerProcessInterface::run(false); // Start hidden

    // List examples directory
    example_files()?.iter().try_for_each(|path| {
        let path = path.to_path_buf();
        viewer.send_request(ViewerRequest::ShowSourceCodeFromFile { path })?;

        prompt_for_confirmation("Was the file loaded?").expect("Invalid source code");

        Ok(())
    })
}

fn main() -> miette::Result<()> {
    use std::env;
    type Test<'a> = (&'a str, fn() -> miette::Result<()>);

    // A single source of truth for test names and functions
    let tests: &[Test] = &[
        ("minimize_restore", test_minimize_restore),
        ("code_from_file", test_code_from_file),
    ];

    let args: Vec<_> = env::args().skip(1).collect(); // skip program name

    if args.is_empty() {
        // Run all tests
        for (_, func) in tests {
            func()?;
        }
        return Ok(());
    }

    // Run only specified tests
    for arg in args {
        let Some((_, func)) = tests.iter().find(|(name, _)| *name == arg) else {
            return Err(miette::miette!("Unknown test: {arg}"));
        };
        func()?;
    }

    Ok(())
}
