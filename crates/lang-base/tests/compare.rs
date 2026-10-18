// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{LineCol, Source, SrcRef, TextEdit, ToHash};

#[test]
fn test_identical_strings_yield_no_edits() {
    let edits = Source::from("hello world\nthis is a test")
        .compare(&Source::from("hello world\nthis is a test"));

    assert!(edits.is_empty(), "Expected no edits for identical strings");
}

#[test]
fn test_pure_insertion() {
    let old = Source::from("hello world");
    let new = Source::from("hello beautiful world");
    let edits = old.compare(&new);

    assert_eq!(edits.len(), 1);
    assert_eq!(
        edits[0],
        TextEdit {
            src_ref: SrcRef::new(&(6..6), LineCol { line: 1, col: 6 }, old.code.to_hash()),
            new_text: "beautiful ".to_string(),
        }
    );
}

#[test]
fn test_pure_deletion() {
    let old = Source::from("hello beautiful world");
    let new = Source::from("hello world");
    let edits = old.compare(&new);

    assert_eq!(edits.len(), 1);
    assert_eq!(
        edits[0],
        TextEdit {
            // Should delete "beautiful " spanning from char 6 to 16
            src_ref: SrcRef::new(&(6..16), LineCol { line: 1, col: 6 }, old.code.to_hash()),
            new_text: String::new(),
        }
    );
}

#[test]
fn test_multiline_replacement() {
    let old = Source::from("line one\nline two\nline three");
    let new = Source::from("line one\nline changed\nline three");
    let edits = old.compare(&new);

    // Depending on how `dissimilar` slices it, this might show up as
    // a deletion of "two" and an insertion of "changed".
    assert_eq!(edits.len(), 2);

    // 1. Deletion of "two"
    assert_eq!(
        edits[0],
        TextEdit {
            // Should delete "beautiful " spanning from char 6 to 16
            src_ref: SrcRef::new(&(14..17), LineCol { line: 2, col: 5 }, old.code.to_hash()),
            new_text: String::new(),
        }
    );

    // 2. Insertion of "changed"
    assert_eq!(
        edits[1],
        TextEdit {
            // Should delete "beautiful " spanning from char 6 to 16
            src_ref: SrcRef::new(&(17..17), LineCol { line: 1, col: 8 }, old.code.to_hash()),
            new_text: "changed".to_string()
        }
    );
}

#[test]
fn test_complex_mixed_changes() {
    let old = Source::from("fn main() {\n    println!(\"Hello\");\n}");
    let new = Source::from("fn main() {\n    // Greet\n    println!(\"Hello World\");\n}");
    let edits = old.compare(&new);

    // We expect edits that insert the comment and modify the string literal
    assert!(!edits.is_empty());

    // A robust way to verify complex edits is to ensure they don't panic
    // and target reasonable line boundaries.
    for edit in &edits {
        // Ensure no inverted ranges
        assert!(edit.src_ref.start <= edit.src_ref.end);

        // Check that bounds stay within the original file lines (0 to 2)
        assert!(edit.src_ref.at.line <= 2);
    }
}
