// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MdBook tests

use mucad_lang_markdown::{MdBook, MdBookError};

/// A markdown file, which cannot be parsed, is reported with its path.
#[test]
fn mdbook_load_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("book.toml"), "[book]\ntitle = \"Test\"\n").expect("book.toml");
    std::fs::create_dir(dir.path().join("src")).expect("src");
    std::fs::write(
        dir.path().join("src/broken.md"),
        "# Broken\n\n```mu,invalid#name\n```\n",
    )
    .expect("broken.md");

    match MdBook::load(dir.path()) {
        Err(MdBookError::Load { file, .. }) => assert!(file.ends_with("src/broken.md")),
        Err(err) => panic!("Unexpected error: {err}"),
        Ok(_) => panic!("A load error"),
    }
}
