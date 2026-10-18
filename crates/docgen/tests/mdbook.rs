// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the mdbook generators.

use std::path::Path;

use mucad_docgen::{BuiltinLibraryDocGenerator, GenerateDoc, LibraryDocGenerator};
use mucad_lang_markdown::{Block, ListItem, MdBook, UnorderedList};
use mucad_lang_resolve::ResolveContext;

/// Display of a markdown file of the book.
fn md_file(mdbook: &MdBook, path: &str) -> String {
    mdbook
        .md_files
        .get(Path::new(path))
        .unwrap_or_else(|| panic!("No file `{path}` in {:?}", mdbook.md_files.keys()))
        .to_string()
}

#[test]
fn builtin_mdbook() {
    let registry = mucad_builtin::BuiltinLibrary::new();
    let mdbook = BuiltinLibraryDocGenerator::new(&registry)
        .generate_doc()
        .expect("No error");

    // Paths are relative to the source folder.
    let summary = md_file(&mdbook, "SUMMARY.md");
    assert!(
        summary.starts_with("# Summary\n\n- [`__mu`](README.md)\n  - [`core`](core/README.md)\n")
    );
    assert!(summary.contains("    - [`Circle`](geo2d/Circle.md)\n"));

    let geo2d = mdbook
        .md_files
        .get(Path::new("geo2d/README.md"))
        .expect("A module page");
    let [Block::UnorderedList(UnorderedList(primitives))] = geo2d.sections()[1].content.as_slice()
    else {
        panic!("A list of primitives");
    };
    assert_eq!(
        primitives[..2],
        [
            ListItem::new("[`Circle`](Circle.md): A circle with a radius."),
            ListItem::new("[`Rect`](Rect.md): A rectangle."),
        ]
    );

    assert!(
        md_file(&mdbook, "core/add.md").contains("> `__mu::core::add(lhs: Any, rhs: Any) -> Any`")
    );
}

#[test]
fn library_mdbook() {
    let mut ctx = ResolveContext::new();
    let (lib, issues) =
        mucad_lang_resolve::resolve("tests/test_cases/doc_lib.mu", &mut ctx).expect("No error");
    assert!(!issues.has_errors(), "{issues:?}");

    let mdbook = LibraryDocGenerator::new(&lib)
        .generate_doc()
        .expect("No error");

    let mut files: Vec<_> = mdbook
        .md_files
        .keys()
        .map(|path| path.display().to_string())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "Disc.md",
            "README.md",
            "Raise.md",
            "SUMMARY.md",
            "sub/Plate.md",
            "sub/README.md"
        ]
    );

    let summary = md_file(&mdbook, "SUMMARY.md");
    assert_eq!(
        summary,
        r#"# Summary

- [`Library`]()
  - [`doc_lib`](README.md)
    - [`sub`](sub/README.md)
      - [`Plate`](sub/Plate.md)
    - [`Disc`](Disc.md)
    - [`Raise`](Raise.md)
"#
    );

    let readme = md_file(&mdbook, "README.md");
    assert!(
        readme.contains("## Sub-modules\n\n- [`sub`](./sub): A sub module.\n"),
        "{readme}"
    );
    assert!(
        readme.contains("## Sketches\n\n- [`Disc`](./Disc.md): A disc with a radius.\n"),
        "{readme}"
    );
    assert!(
        readme.contains("## Operations\n\n- [`Raise`](./Raise.md): Extrude a disc.\n"),
        "{readme}"
    );
    assert!(
        readme.contains("## Constants\n\n- `R` = `4mm`\n"),
        "{readme}"
    );
}

/// The book can be written and contains `book.toml` and the markdown files below `src`.
#[test]
fn write_builtin_mdbook() {
    let dir = std::env::temp_dir().join("mucad-docgen-test-builtin");
    let _ = std::fs::remove_dir_all(&dir);

    let registry = mucad_builtin::BuiltinLibrary::new();
    BuiltinLibraryDocGenerator::new(&registry)
        .write_doc(&dir)
        .expect("No error");

    assert!(dir.join("book.toml").is_file());
    assert!(dir.join("src/SUMMARY.md").is_file());
    assert!(dir.join("src/geo2d/Circle.md").is_file());

    std::fs::remove_dir_all(&dir).expect("Clean up");
}
