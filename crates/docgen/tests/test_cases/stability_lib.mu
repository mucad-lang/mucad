/// The new function.
pub fn g() {
    return 1;
}

/// The old function.
#[deprecated(since = "0.5.0", note = "Use `g()` instead", removal_in = "0.7.0")]
pub fn f() {
    return g();
}

/// An experimental sketch.
#[experimental]
#[introduced(version = "0.6.0")]
pub sketch Gear(radius: Length, teeth = 12) {
    /// With the diameter instead of the radius.
    init(diameter: Length, teeth = 12) {
        radius = diameter / 2;
    }

    /// With the old name of the radius.
    #[deprecated(note = "Use `radius`")]
    init(r: Length) {
        radius = r;
    }

    __mu::geo2d::Circle(radius = radius);
}

#[deprecated]
pub const OLD = 1;
