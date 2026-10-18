// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Common exporter test functions

use std::path::PathBuf;

use mucad_builtin::{BuiltinEvalContext, BuiltinPrimitiveCall, mu};
use mucad_core::Rect;
use mucad_lang_types::{Model, Value, arguments};

pub fn circle(r: f64) -> Model {
    let mut ctx = BuiltinEvalContext::default();
    mu::geo2d::Circle::call(arguments!(radius = Value::mm(r)), &mut ctx).expect("No error")
}

pub fn rect(rect: Rect) -> Model {
    let mut ctx = BuiltinEvalContext::default();
    let (width, height) = (rect.width(), rect.height());
    let (x, y) = rect.min().x_y();
    mu::geo2d::Rect::call(
        arguments!(
            x = Value::mm(x),
            y = Value::mm(y),
            width = Value::mm(width),
            height = Value::mm(height)
        ),
        &mut ctx,
    )
    .expect("No error")
}

/// Resolves the root workspace directory using Cargo's metadata.
pub fn workspace_dir() -> PathBuf {
    cargo_metadata::MetadataCommand::new()
        .exec()
        .map(|metadata| metadata.workspace_root.into_std_path_buf())
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

/// Resolves the target directory using Cargo's metadata.
pub fn target_dir() -> PathBuf {
    cargo_metadata::MetadataCommand::new()
        .exec()
        .map(|metadata| metadata.target_directory.into_std_path_buf())
        .unwrap_or_else(|_| {
            std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace_dir().join("target"))
        })
}
