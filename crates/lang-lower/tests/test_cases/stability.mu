// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// A stable constant.
#[stable]
pub const STABLE = 1;

/// A deprecated constant.
#[deprecated]
pub const DEPRECATED = 2;

/// An experimental constant.
#[experimental]
pub const EXPERIMENTAL = 3;

/// A deprecated function with all arguments.
#[deprecated(since = "0.5.0", note = "Use g() instead", removal_in = "0.7.0")]
pub fn f() {
    return 1;
}

/// An experimental function introduced in 0.6.0.
#[experimental]
#[introduced(version = "0.6.0")]
pub fn g() {
    return 2;
}

/// A deprecated module.
#[deprecated(since = "0.6.0")]
pub mod old {
    pub const A = 1;
}

/// An experimental sketch.
#[experimental]
pub sketch Disc(radius: Length) {
    /// A deprecated initializer.
    #[deprecated(note = "Use the diameter")]
    init(r: Length) {
        radius = r;
    }

    /// An experimental initializer.
    #[experimental]
    init(diameter: Length) {
        radius = diameter / 2;
    }

    /// A deprecated property.
    #[deprecated(note = "Use radius")]
    prop r = radius;

    /// An experimental property.
    /// Twice the radius.
    #[experimental]
    prop d = radius * 2;

    __mu::geo2d::Circle(radius);
}
