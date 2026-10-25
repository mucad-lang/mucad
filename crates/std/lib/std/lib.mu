// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

pub mod geo2d;
pub mod geo3d;
pub mod list;
pub mod math;
pub mod ops;
pub mod string;

// Check a condition and stop with an error message if it does not hold.
pub use __mu::debug::assert;
pub use __mu::debug::assert_eq;
pub use __mu::debug::expect;
pub use __mu::debug::panic;

pub use __mu::debug::error;
pub use __mu::debug::warning;
pub use __mu::debug::info;
pub use __mu::debug::print;
