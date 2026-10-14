// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Code block tests

use std::str::FromStr;

use microcad_lang_markdown::{CodeBlockHeader, Markdown};

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

/// Test banners are inserted, overwritten if wrong and kept if right.
#[test]
fn set_test_banners() {
    let banner = CodeBlockHeader::test_banner_string;
    let mut md = Markdown::from_str(&format!(
        "# Test\n\n\
        ```mu,a\n```\n\n\
        {right}\n\n```mu,b\n```\n\n\
        {wrong}\n\n```mu,c\n```\n\n\
        ```mu\n```\n",
        right = banner("b"),
        wrong = banner("wrong"),
    ))
    .expect("Markdown");

    let set_banners = |md: &mut Markdown| {
        md.set_test_banners(|code_block| {
            code_block.name().as_ref().map(|name| banner(name.as_str()))
        })
    };

    assert!(set_banners(&mut md));
    let text = md.to_string();
    for name in ["a", "b", "c"] {
        assert_eq!(text.matches(&banner(name)).count(), 1, "{text}");
    }
    assert!(!text.contains("wrong"), "{text}");

    // Nothing changes, if all banners are right.
    assert!(!set_banners(&mut md));
    assert_eq!(md.to_string(), text);
}
