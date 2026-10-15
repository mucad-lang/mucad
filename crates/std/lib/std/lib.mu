// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod geo2d;
pub mod geo3d;
pub mod ops;
pub mod math;
pub mod list;

// Check a condition and stop with an error message if it does not hold.
pub use __mu::debug::assert;
pub use __mu::debug::assert_eq;
pub use __mu::debug::expect;
pub use __mu::debug::panic;

pub use __mu::debug::error;
pub use __mu::debug::warning;
pub use __mu::debug::info;
pub use __mu::debug::print;
