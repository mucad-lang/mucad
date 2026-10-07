// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
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
    let (lib, _diags) = microcad_lang_resolve::resolve("tests/test_cases/resolve.µcad", &mut ctx)
        .expect("No error");

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
        microcad_lang_resolve::resolve("tests/test_cases/single_identifier.µcad", &mut ctx)
            .expect("No error");

    assert!(!issues.has_errors(), "{issues:?}");
}

/// An unknown identifier is reported as unresolved symbol instead of panicking.
#[test]
fn resolve_unresolved_path() {
    let mut ctx = ResolveContext::new();
    let (_lib, issues) =
        microcad_lang_resolve::resolve("tests/test_cases/unresolved_path.µcad", &mut ctx)
            .expect("No fatal error");

    assert!(
        issues
            .iter()
            .any(|issue| issue.to_string() == "Unresolved symbol: undefined_radius"),
        "{issues:?}"
    );
}
