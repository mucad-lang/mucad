// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the mdbook generators.

use std::path::Path;

use microcad_docgen::{BuiltinLibraryDocGenerator, GenerateDoc, LibraryDocGenerator};
use microcad_lang_markdown::{Block, ListItem, MdBook, UnorderedList};
use microcad_lang_resolve::ResolveContext;

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
    let registry = microcad_builtin::BuiltinLibrary::new();
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
    assert_eq!(
        geo2d.sections()[1].content,
        [Block::UnorderedList(UnorderedList::new([
            ListItem::new("[`Circle`](Circle.md): A circle with a radius."),
            ListItem::new("[`Rect`](Rect.md): A rectangle."),
        ]))]
    );

    assert!(
        md_file(&mdbook, "core/add.md").contains("> `__mu::core::add(lhs: Any, rhs: Any) -> Any`")
    );
}

#[test]
fn library_mdbook() {
    let mut ctx = ResolveContext::new();
    let (lib, issues) = microcad_lang_resolve::resolve("tests/test_cases/doc_lib.µcad", &mut ctx)
        .expect("No error");
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
        "# Summary\n\n\
         - [`doc_lib`](README.md)\n  \
           - [`sub`](sub/README.md)\n    \
             - [`Plate`](sub/Plate.md)\n  \
           - [`Disc`](Disc.md)\n  \
           - [`Raise`](Raise.md)\n"
    );

    let readme = md_file(&mdbook, "README.md");
    assert!(
        readme.contains("## Sub-modules\n\n- [`sub`](./sub/README.md): A sub module.\n"),
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
    let dir = std::env::temp_dir().join("microcad-docgen-test-builtin");
    let _ = std::fs::remove_dir_all(&dir);

    let registry = microcad_builtin::BuiltinLibrary::new();
    BuiltinLibraryDocGenerator::new(&registry)
        .write_doc(&dir)
        .expect("No error");

    assert!(dir.join("book.toml").is_file());
    assert!(dir.join("src/SUMMARY.md").is_file());
    assert!(dir.join("src/geo2d/Circle.md").is_file());

    std::fs::remove_dir_all(&dir).expect("Clean up");
}
