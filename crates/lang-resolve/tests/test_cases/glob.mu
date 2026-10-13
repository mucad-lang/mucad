pub mod geo {
    pub sketch Shape() {
        __mu::geo2d::Circle(radius = 1mm);
    }

    pub mod inner {
        pub const R = 1mm;
    }
}

use geo::*;

// Imported by the glob.
Shape();

// Glob imports are visible in nested scopes and allow further path parts.
pub sketch Uses() {
    Shape();
    __mu::geo2d::Circle(radius = inner::R);
}
