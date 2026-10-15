// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resolve the intermediate representation into a [`Symbol`].

mod resolve;

mod diag;

mod source_unit;

pub mod locate;

pub use source_unit::SourceUnit;

pub use diag::{
    ResolveError, ResolveErrorKind, ResolveInfo, ResolveIssue, ResolveResult, ResolveWarning,
};

pub use resolve::{
    LibraryCache, Prelude, ResolveContext, ResolveLibraryContext, SourceCache,
    library_source_files, resolve, resolve_source,
};

pub mod library;

pub use library::{
    ExternalLookUp, Library, LookUp, Manifest, ManifestError,
    symbol::{SymbolDef, SymbolId, SymbolNode, SymbolNodeExt, SymbolNodeMut, SymbolNodeRef},
};

#[macro_export]
macro_rules! argument_list {
    ( $( $key:ident = $val:expr ),* $(,)? ) => {
        $crate::library::symbol::ArgumentList::from_iter(
        [
            $(
                $crate::library::symbol::Argument::named(stringify!($key), $val)
            ),*
        ])
    };
}

#[macro_export]
macro_rules! call_builtin {
    // Matches any path (e.g. core::mul or core::math::mul) followed by standard argument syntax
    ( $module:ident::$name:ident ( $( $args:tt )* ) ) => {
        $crate::library::symbol::Call::builtin(__mu!( $module::$name ))
            .with_args($crate::argument_list!( $( $args )* ))
    };
}

/// Construct an that is guaranteed to be convertable via `.into()` into an expression.
#[macro_export]
macro_rules! expr {
    // A literal without unit
    ($lit:literal) => {
        Value::from($lit)
    };
    // Angle in degrees
    ($lit:literal deg) => {
        Value::deg($lit)
    };
    // Length in millimeter
    ($lit:literal mm) => {
        Value::mm($lit)
    };
    // A local identifier
    ($id:ident) => {
        symbol::Path::Resolved(SymbolId::Local(stringify!($id).into()))
    };
    ($id:ident.$field:ident) => {
        call_builtin!(core::member_access(
            lhs = expr!($id),
            name = Value::from(stringify!($field))
        ))
    };
}
