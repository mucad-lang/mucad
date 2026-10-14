// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod geo2d;
pub mod geo3d;
pub mod ops;
pub mod math;

// Check a condition and stop with an error message if it does not hold.
pub use __mu::debug::assert;

pub use __mu::debug::print;

/*
pub use __mu::math;
pub use __mu::print;
pub use __mu::type_of;
pub use __mu::import;
pub use __mu::debug;
pub use __mu::log;
pub use __mu::color;
pub use __mu::to_string;
pub use __mu::list;
pub use __mu::string;
*/
