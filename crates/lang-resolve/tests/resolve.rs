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

/// A glob import (`use geo::*;`) makes the public symbols of a module available.
#[test]
fn resolve_glob() {
    let mut ctx = ResolveContext::new();
    let (_lib, issues) =
        microcad_lang_resolve::resolve("tests/test_cases/glob.mu", &mut ctx).expect("No error");

    assert!(!issues.has_errors(), "{issues:?}");
}

/// A glob import of a module of an external library, e.g. `use std::geo2d::*;`.
#[test]
fn resolve_glob_external() {
    let mut ctx = ResolveContext::new();
    let (lib_id, issues) = ctx
        .load_external_library("tests/test_cases/glob_lib")
        .expect("A library");
    assert!(!issues.has_errors(), "{issues:?}");
    ctx.add_implicit_dependency("globlib", lib_id);

    let (_lib, issues) = microcad_lang_resolve::resolve_source(
        microcad_lang_base::Source::from(
            r#"
            use globlib::geo::*;

            Shape();
            __mu::geo2d::Circle(radius = R);

            sketch Uses() {
                Shape();
            }
            "#,
        ),
        &mut ctx,
    )
    .expect("No fatal error");

    assert!(!issues.has_errors(), "{issues:?}");
}

/// Private symbols are not imported by a glob import.
#[test]
fn resolve_glob_external_private() {
    let mut ctx = ResolveContext::new();
    let (lib_id, _) = ctx
        .load_external_library("tests/test_cases/glob_lib")
        .expect("A library");
    ctx.add_implicit_dependency("globlib", lib_id);

    let (_lib, issues) = microcad_lang_resolve::resolve_source(
        microcad_lang_base::Source::from("use globlib::geo::*;\nPrivate();\n"),
        &mut ctx,
    )
    .expect("No fatal error");

    assert!(
        issues
            .iter()
            .any(|issue| issue.to_string() == "Unresolved symbol: Private"),
        "{issues:?}"
    );
}

/// A public glob import re-exports the symbols of a module (`pub use b::*;`).
#[test]
fn resolve_glob_reexport() {
    let mut ctx = ResolveContext::new();
    let (_lib, issues) = microcad_lang_resolve::resolve_source(
        microcad_lang_base::Source::from(
            r#"
            pub mod a {
                pub use b::*;
            }
            pub mod b {
                pub const C = 1mm;
            }

            use a::*;
            __mu::geo2d::Circle(radius = C);
            "#,
        ),
        &mut ctx,
    )
    .expect("No fatal error");

    assert!(!issues.has_errors(), "{issues:?}");
}

/// A private glob import does not re-export the symbols of a module.
#[test]
fn resolve_glob_private_no_reexport() {
    let mut ctx = ResolveContext::new();
    let (_lib, issues) = microcad_lang_resolve::resolve_source(
        microcad_lang_base::Source::from(
            r#"
            pub mod a {
                use b::*;
            }
            pub mod b {
                pub const C = 1mm;
            }

            use a::*;
            __mu::geo2d::Circle(radius = C);
            "#,
        ),
        &mut ctx,
    )
    .expect("No fatal error");

    assert!(
        issues
            .iter()
            .any(|issue| issue.to_string() == "Unresolved symbol: C"),
        "{issues:?}"
    );
}
