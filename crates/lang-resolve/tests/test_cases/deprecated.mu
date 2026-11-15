// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The new function.
pub fn g() {
    return 1;
}

#[deprecated(since = "0.5.0", note = "Use g() instead")]
pub fn f() {
    return g();
}

// The old name of `g`.
#[deprecated]
pub use g as h;

#[deprecated(since = "0.6.0")]
pub mod old {
    pub const A = 1;

    // Inside a deprecated module, deprecated symbols are not reported.
    pub fn uses_f() {
        return super::f();
    }
}

#[deprecated]
pub const B = 2;

// Inside a deprecated function, deprecated symbols are not reported.
#[deprecated]
pub fn uses_deprecated() {
    return f() + h() + old::A;
}

pub fn uses() {
    a = f(); // warning: resolve::deprecated
    b = h(); // warning: resolve::deprecated
    c = old::A; // warning: resolve::deprecated
    return a + b + c + g();
}

// The default value is also one of the implicit init, but reported once.
pub sketch Disc(radius: Length = B * 1mm) { // warning: resolve::deprecated
    __mu::geo2d::Circle(radius);
}
