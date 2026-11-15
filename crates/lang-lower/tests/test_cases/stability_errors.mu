// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

#[stable]
#[experimental] // error: mucad::lower::conflicting_stability
pub const A = 1;

#[introduced(version = "0.5.0")]
#[introduced(version = "0.6.0")] // error: mucad::lower::duplicate_attribute
pub const B = 2;

#[introduced(version = "zero")] // error: mucad::lower::invalid_version_string
pub const C = 3;

#[introduced] // error: mucad::lower::missing_attribute_argument
pub const D = 4;

#[deprecated(since = "soon")] // error: mucad::lower::invalid_version_string
pub fn e() {
    return 5;
}

#[deprecated(removal_in = "later")] // error: mucad::lower::invalid_version_string
pub fn f() {
    return 6;
}

pub sketch S(a: Length) {
    #[experimental]
    #[deprecated] // error: mucad::lower::conflicting_stability
    init(b: Length) {
        a = b;
    }

    #[stable]
    #[deprecated(note = "Use a")] // error: mucad::lower::conflicting_stability
    prop p = a;

    __mu::geo2d::Circle(radius = a);
}
