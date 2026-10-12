// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! resolve tests

use std::str::FromStr;

use microcad_lang_lower::ir::UnresolvedPath;
use microcad_lang_resolve::{ResolveContext, SymbolNodeRef};

fn path(name: &str) -> UnresolvedPath {
    UnresolvedPath::from_str(name).expect("A valid path")
}

#[test]
fn resolve() {
    let mut ctx = ResolveContext::new();
    let (lib, _diags) =
        microcad_lang_resolve::resolve("tests/test_cases/resolve.mu", &mut ctx).expect("No error");

    let symbol = lib
        .look_up(lib.root, &path("::resolve"))
        .map(|id| SymbolNodeRef::new(id, &lib.arena))
        .expect("A node");

    insta::assert_snapshot!("resolve", &symbol)
}

/// Single identifiers resolve to constants, workbenches, parameters and locals.
#[test]
fn resolve_single_identifier() {
    let mut ctx = ResolveContext::new();
    let (_lib, issues) =
        microcad_lang_resolve::resolve("tests/test_cases/single_identifier.mu", &mut ctx)
            .expect("No error");

    assert!(!issues.has_errors(), "{issues:?}");
}

/// A library appears in its own tree under its name and can refer to itself, despite `no_std`.
#[test]
fn resolve_self_reference() {
    let mut ctx = ResolveContext::new();
    let (lib, issues) = microcad_lang_resolve::resolve("tests/test_cases/self_reference", &mut ctx)
        .expect("No fatal error");

    assert!(!issues.has_errors(), "{issues:?}");
    assert!(
        lib.look_up(lib.root, &path("::selfref::geo::Shape"))
            .is_some()
    );
}

/// A file with an unknown built-in is still loaded and the built-in is reported once.
#[test]
fn resolve_unknown_builtin() {
    let mut ctx = ResolveContext::new();
    let (lib, issues) =
        microcad_lang_resolve::resolve("tests/test_cases/unknown_builtin.mu", &mut ctx)
            .expect("No fatal error");

    let errors: Vec<_> = issues.iter().map(|issue| issue.to_string()).collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("No built-in `__mu::geo2d::Unknown`"),
        "{errors:?}"
    );

    // The rest of the file is loaded.
    assert!(
        lib.look_up(lib.root, &path("::unknown_builtin::Known"))
            .is_some()
    );
    assert!(
        lib.look_up(lib.root, &path("::unknown_builtin::Missing"))
            .is_some()
    );
}

/// An unknown identifier is reported as unresolved symbol instead of panicking.
#[test]
fn resolve_unresolved_path() {
    let mut ctx = ResolveContext::new();
    let (_lib, issues) =
        microcad_lang_resolve::resolve("tests/test_cases/unresolved_path.mu", &mut ctx)
            .expect("No fatal error");

    assert!(
        issues
            .iter()
            .any(|issue| issue.to_string() == "Unresolved symbol: undefined_radius"),
        "{issues:?}"
    );
}
