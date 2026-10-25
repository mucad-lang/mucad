// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Code block tests

use mucad_lang_markdown::Markdown;

/// The source of a code block in a loaded markdown file has the location of the code block.
#[test]
fn code_block_source_location() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("test.md");
    std::fs::write(
        &path,
        "# Test\n\nText\n\n```mu,circle\nCircle(r = 1mm);\n```\n",
    )
    .expect("test.md");

    let md = Markdown::load(&path).expect("Markdown");
    let code_block = md.code_blocks().next().expect("A code block");
    assert_eq!(code_block.path.as_deref(), Some(path.as_path()));

    let source = code_block.source();
    assert_eq!(source.path(), Some(path.clone()));
    assert_eq!(source.location.line_offset, Some(5));
}
