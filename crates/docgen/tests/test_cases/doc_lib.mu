/// A disc with a radius.
pub sketch Disc(radius: Length) {
    __mu::geo2d::Circle(radius = radius);
}

/// Extrude a disc.
pub op Raise(height: Length) {
    __mu::ops::extrude(self = @input, height = height);
}

pub const R = 4mm;

/// A sub module.
pub mod sub {
    /// A square plate.
    pub part Plate(size: Length) {
        __mu::ops::extrude(
            self = __mu::geo2d::Rect(width = size, height = size, x = 0mm, y = 0mm),
            height = 1mm, scale_x = 1.0, scale_y = 1.0, twist = 0°
        );
    }
}
