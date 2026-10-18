// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::mu::*;
use mucad_macros::test_builtin_fn;

// --- Debug & Diagnostic Functions ---

#[test]
#[test_builtin_fn(debug::assert(cond = true, message = "ok") == ())]
fn assert_pass() {}

#[test]
#[test_builtin_fn(debug::expect(cond = true, message = "ok") == ())]
#[test_builtin_fn(debug::expect(cond = false, message = "warning") == ())]
fn expect() {}

#[test]
#[test_builtin_fn(debug::error(message = "custom error") == ())]
fn debug_error() {}

#[test]
#[test_builtin_fn(debug::warning(message = "custom warning") == ())]
fn debug_warning() {}

#[test]
#[test_builtin_fn(debug::info(message = "custom info") == ())]
fn debug_info() {}
