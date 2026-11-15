// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

pub mod geo2d {
    /// A circle with an experimental initializer.
    pub sketch Circle(radius: Length) {
        #[experimental]
        init(diameter: Length) {
            radius = diameter / 2;
        }

        __mu::geo2d::Circle(radius);
    }

    /// An experimental sketch.
    #[experimental]
    pub sketch Gear(radius: Length) {
        __mu::geo2d::Circle(radius);
    }
}
