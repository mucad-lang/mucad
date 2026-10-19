// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests of the symbols to complete a path.

use std::str::FromStr;

use mucad_lang_base::{Source, element::WorkbenchKind};
use mucad_lang_lower::ir::UnresolvedPath;
use mucad_lang_resolve::{Completion, CompletionKind, Prelude, ResolveContext};

/// A context with the library `globlib` as dependency and its constant `R` in the prelude.
fn context() -> ResolveContext {
    let mut ctx = ResolveContext::new();
    let (lib_id, issues) = ctx
        .load_external_library("tests/test_cases/glob_lib")
        .expect("A library");
    assert!(!issues.has_errors(), "{issues:?}");
    ctx.add_implicit_dependency("globlib", lib_id);

    let lib = ctx
        .lib_cache()
        .with_read(|lib_cache| lib_cache.get(&lib_id))
        .expect("The library");
    let r = lib
        .look_up(lib.root, &path("::globlib::geo::R"))
        .expect("A constant");
    let mut prelude = Prelude::new(lib_id);
    prelude
        .symbols
        .insert("R".into(), lib.symbol_id(r, Some(lib_id)));
    ctx.set_prelude(prelude);
    ctx
}

fn path(path: &str) -> UnresolvedPath {
    UnresolvedPath::from_str(path).expect("A path")
}

/// The completions of `qualifier` in `code` at the position of `|`.
fn complete(code: &str, qualifier: &str) -> Vec<Completion> {
    let offset = code.find('|').expect("A position");
    let source = Source::from(code.replacen('|', "", 1).as_str());

    let mut ctx = context();
    let (lib, issues) =
        mucad_lang_resolve::resolve_source(source.clone(), &mut ctx).expect("A library");
    assert!(!issues.has_errors(), "{issues:?}");
    ctx.lib_cache()
        .with_write(|lib_cache| lib_cache.insert_root(lib));

    ctx.completions(&source, offset, &path(qualifier))
}

fn names(completions: &[Completion]) -> Vec<&str> {
    completions.iter().map(|c| c.name.as_str()).collect()
}

const CODE: &str = r#"
use globlib::geo::*;

/// The top sketch.
sketch Top() {}

const A = 1mm;

mod inner {
    pub sketch Inner() {}
    const B = 2mm;
    @inner
}

@root
"#;

/// The code with `|` at `marker`, e.g. `@root`.
fn code_at(marker: &str) -> String {
    CODE.replace(marker, "|")
        .replace("@inner", "")
        .replace("@root", "")
}

/// Visible names: symbols, imported symbols, libraries and the prelude, but no private symbols
/// of other modules.
#[test]
fn complete_names() {
    let completions = complete(&code_at("@root"), "");
    assert_eq!(
        names(&completions),
        ["A", "R", "Shape", "Top", "globlib", "inner"]
    );

    let kind = |name: &str| {
        completions
            .iter()
            .find(|c| c.name.as_str() == name)
            .map(|c| c.kind)
    };
    assert_eq!(kind("A"), Some(CompletionKind::Constant));
    assert_eq!(
        kind("Top"),
        Some(CompletionKind::Workbench(WorkbenchKind::Sketch))
    );
    assert_eq!(kind("inner"), Some(CompletionKind::Module));
    assert_eq!(kind("globlib"), Some(CompletionKind::Library));

    let top = completions.iter().find(|c| c.name.as_str() == "Top");
    assert_eq!(top.and_then(|c| c.doc.as_deref()), Some("The top sketch."));
}

/// Inside a module, its private symbols and those of the parent scopes are visible.
#[test]
fn complete_names_in_module() {
    let completions = complete(&code_at("@inner"), "");
    assert_eq!(
        names(&completions),
        ["A", "B", "Inner", "R", "Shape", "Top", "globlib", "inner"]
    );
}

/// The prelude completes names without `use`.
#[test]
fn complete_prelude() {
    assert_eq!(names(&complete("|", "")), ["R", "globlib"]);
}

/// Inside a module, only its public symbols are visible from the outside.
#[test]
fn complete_module() {
    assert_eq!(names(&complete(&code_at("@root"), "inner")), ["Inner"]);
    assert_eq!(
        names(&complete(&code_at("@inner"), "super::inner")),
        ["B", "Inner"]
    );
}

/// The public symbols of a module of a library.
#[test]
fn complete_library_module() {
    assert_eq!(names(&complete("|", "globlib")), ["geo"]);
    assert_eq!(names(&complete("|", "globlib::geo")), ["R", "Shape"]);
}

/// Built-ins and unknown modules are not completed.
#[test]
fn complete_nothing() {
    assert!(complete("|", "__mu").is_empty());
    assert!(complete("|", "__mu::geo2d").is_empty());
    assert!(complete("|", "unknown").is_empty());
}

/// With a library of another version of the source, e.g. if the source has syntax errors,
/// the names of the source file are completed.
#[test]
fn complete_names_in_other_source_version() {
    let mut ctx = context();
    let (lib, _) =
        mucad_lang_resolve::resolve_source(Source::from(CODE), &mut ctx).expect("A library");
    ctx.lib_cache()
        .with_write(|lib_cache| lib_cache.insert_root(lib));

    let edited = Source::from(format!("{CODE}\nTo").as_str());
    let completions = ctx.completions(&edited, edited.code().len(), &path(""));
    assert_eq!(
        names(&completions),
        ["A", "R", "Shape", "Top", "globlib", "inner"]
    );
}
