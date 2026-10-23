// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Library tests

use std::str::FromStr;

use mucad_lang_base::{DiagRenderOptions, Diagnostics};
use mucad_lang_lower::ir::UnresolvedPath;
use mucad_lang_resolve::{Library, ResolveContext, ResolveLibraryContext, SymbolNodeExt};

fn lib_ctx<'lib, 'ctx>(
    lib: &'lib mut Library,
    ctx: &'ctx mut ResolveContext,
) -> ResolveLibraryContext<'lib, 'ctx>
where
    'lib: 'ctx,
    'ctx: 'lib,
{
    ctx.load_external_library("../../crates/std/lib/std")
        .expect("No error");
    ResolveLibraryContext::new(lib, ctx)
}

fn path(name: &str) -> UnresolvedPath {
    UnresolvedPath::from_str(name).expect("A valid path")
}

#[test]
fn load_source_inline_module() {
    let mut library = Library::new();
    let mut ctx = ResolveContext::new();
    let mut lib_ctx = lib_ctx(&mut library, &mut ctx);

    match lib_ctx.load_file("tests/test_cases/inline_module.mu") {
        Ok(_) => {
            let library = lib_ctx.lib();
            for child in library.root().children() {
                println!("{child}");
            }
            let root = library.root();
            let inline_module = root.find_symbol_node("inline_module").expect("A child");

            let b = library
                .look_up(inline_module.id, &path("b"))
                .expect("Add node");
            library
                .look_up(b, &path("::inline_module"))
                .expect("An inline module");

            let d = library
                .look_up(b, &path("::inline_module::b::d"))
                .expect("An inline module 'd'");

            let _c = library
                .look_up(b, &path("::inline_module::b::C"))
                .expect("A constant 'C'");

            // Search `b` in `d`
            let _b = library
                .look_up(d, &path("b"))
                .expect("An inline module 'b'");

            insta::assert_snapshot!("load_source_inline_module", library);
        }
        Err(err) => {
            let diagnostics = Diagnostics::from(lib_ctx.to_issues());

            ctx.source_cache().with_read(|source_cache| {
                panic!(
                    "{diagnostics}\n{err:?}",
                    diagnostics = diagnostics
                        .render_to_string(source_cache, &DiagRenderOptions::default())
                        .unwrap()
                )
            })
        }
    }
}

#[test]
fn load_source_file_module() {
    let mut lib = Library::new();
    let mut ctx = ResolveContext::new();
    let mut lib_ctx = lib_ctx(&mut lib, &mut ctx);

    lib_ctx
        .load_file("tests/test_cases/file_module.mu")
        .expect("No error");

    insta::assert_snapshot!("load_source_file_module", lib)
}

/// The paths of the source files are relative to the directory of the library,
/// so a library can be stored, e.g. in a `.mulib` file, and loaded from anywhere.
#[test]
fn source_files_relative_to_library() {
    use mucad_lang_base::Artifact;
    use std::path::PathBuf;

    let mut ctx = ResolveContext::new();
    let (lib, _) = mucad_lang_resolve::resolve("tests/test_cases/file_module.mu", &mut ctx)
        .expect("A library");
    assert_eq!(lib.dir, PathBuf::from("tests/test_cases"));

    // The source files can be loaded, e.g. to watch them.
    let files = lib.source_files();
    assert!(files.contains(&PathBuf::from("tests/test_cases/file_module/foo.mu")));
    assert!(files.iter().all(|file| file.exists()), "{files:?}");

    // The stored library has no directory.
    let stored = Library::from_binary(&lib.to_binary().expect("Binary")).expect("A library");
    assert_eq!(stored.dir, PathBuf::new());
    assert!(
        stored
            .source_files()
            .contains(&PathBuf::from("file_module/foo.mu"))
    );

    // In its directory, the stored library has the same source files.
    let loaded = Library {
        dir: "tests/test_cases".into(),
        ..stored
    };
    assert_eq!(loaded.source_files(), files);
}
